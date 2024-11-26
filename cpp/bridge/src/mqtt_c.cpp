#include "mqtt_c.hpp"
#include "PolarMqtt.hpp"
#include <unordered_map>
#include <mutex>

// Structure to hold callback context and handlers
struct CallbackContext
{
    mqtt_message_callback_t msg_callback;
    void *msg_context;
    mqtt_state_callback_t state_callback;
    void *state_context;
    std::unique_ptr<mqtt::MessageHandler> msg_handler;
    std::unique_ptr<mqtt::SessionHandler> session_handler;
};

// Global factory instance management
static mqtt::APIFactory *g_factory = nullptr;
static std::mutex g_factory_mutex;

// Session callback mapping
static std::unordered_map<mqtt_session_t *, CallbackContext> g_callbacks;
static std::mutex g_callbacks_mutex;

// Bridge class implementations
class BridgeMessageHandler : public mqtt::MessageHandler
{
private:
    CallbackContext &context;

public:
    BridgeMessageHandler(CallbackContext &ctx) : context(ctx) {}

    void onMessage(const mqtt::Message &message) override
    {
        if (context.msg_callback)
        {
            context.msg_callback(reinterpret_cast<const mqtt_message_t *>(&message),
                                 context.msg_context);
        }
    }
};

class BridgeSessionHandler : public mqtt::SessionHandler
{
private:
    CallbackContext &context;

public:
    BridgeSessionHandler(CallbackContext &ctx) : context(ctx) {}

    void onStateChange(mqtt::SessionState newState) override
    {
        if (context.state_callback)
        {
            context.state_callback(static_cast<mqtt_state_t>(newState),
                                   context.state_context);
        }
    }

    void onError(int errorCode, const char *message) override
    {
        // TODO: Implement error handling
    }
};

// C linkage to all exported functions
extern "C"
{

    // Factory functions
    mqtt_factory_t *mqtt_factory_get_instance(void)
    {
        std::lock_guard<std::mutex> lock(g_factory_mutex);
        if (!g_factory)
        {
            g_factory = mqtt::APIFactory::getInstance();
        }
        return reinterpret_cast<mqtt_factory_t *>(g_factory);
    }

    void mqtt_factory_destroy(mqtt_factory_t *factory)
    {
        std::lock_guard<std::mutex> lock(g_factory_mutex);
        if (factory == reinterpret_cast<mqtt_factory_t *>(g_factory))
        {
            g_factory->uninitialize();
            g_factory = nullptr;
        }
    }

    int mqtt_factory_initialize(const char *app_name, const char *app_version, int debug)
    {
        auto factory = reinterpret_cast<mqtt::APIFactory *>(mqtt_factory_get_instance());
        return factory->initialize(app_name, app_version, debug != 0);
    }

    // Session management
    mqtt_session_t *mqtt_session_create(mqtt_factory_t *factory,
                                        const char *client_id,
                                        mqtt_state_callback_t state_cb,
                                        void *state_context)
    {
        auto cpp_factory = reinterpret_cast<mqtt::APIFactory *>(factory);

        // Create callback context
        CallbackContext ctx = {
            nullptr, nullptr, state_cb, state_context,
            nullptr, nullptr};

        // Create session handler
        ctx.session_handler = std::make_unique<BridgeSessionHandler>(ctx);
        auto &session = cpp_factory->createSession(client_id, *ctx.session_handler);

        // Store session and context
        auto session_ptr = reinterpret_cast<mqtt_session_t *>(&session);
        {
            std::lock_guard<std::mutex> lock(g_callbacks_mutex);
            g_callbacks[session_ptr] = std::move(ctx);
        }

        return session_ptr;
    }

    void mqtt_session_destroy(mqtt_session_t *session)
    {
        if (!session)
            return;

        auto cpp_session = reinterpret_cast<mqtt::Session *>(session);

        // Clean up callbacks
        {
            std::lock_guard<std::mutex> lock(g_callbacks_mutex);
            g_callbacks.erase(session);
        }

        // Destroy session
        mqtt::APIFactory::getInstance()->destroySession(*cpp_session);
    }

    int mqtt_session_start(mqtt_session_t *session)
    {
        if (!session)
            return -1;
        auto cpp_session = reinterpret_cast<mqtt::Session *>(session);
        return cpp_session->start() ? 0 : -1;
    }

    int mqtt_session_stop(mqtt_session_t *session)
    {
        if (!session)
            return -1;
        auto cpp_session = reinterpret_cast<mqtt::Session *>(session);
        return cpp_session->stop() ? 0 : -1;
    }

    mqtt_state_t mqtt_session_get_state(const mqtt_session_t *session)
    {
        if (!session)
            return MQTT_STATE_DISCONNECTED;
        auto cpp_session = reinterpret_cast<const mqtt::Session *>(session);
        return static_cast<mqtt_state_t>(cpp_session->getState());
    }

    // Session operations
    int64_t mqtt_session_subscribe(mqtt_session_t *session,
                                   const char *topic,
                                   mqtt_qos_t qos,
                                   mqtt_message_callback_t msg_cb,
                                   void *msg_context)
    {
        if (!session)
            return -1;

        auto cpp_session = reinterpret_cast<mqtt::Session *>(session);

        // Update callback context
        {
            std::lock_guard<std::mutex> lock(g_callbacks_mutex);
            auto &ctx = g_callbacks[session];
            ctx.msg_callback = msg_cb;
            ctx.msg_context = msg_context;

            // Create and set message handler if needed
            if (msg_cb && !ctx.msg_handler)
            {
                ctx.msg_handler = std::make_unique<BridgeMessageHandler>(ctx);
                cpp_session->setMessageHandler(ctx.msg_handler.get());
            }
        }

        return cpp_session->subscribe(topic, static_cast<mqtt::Message::QoS>(qos));
    }

    int mqtt_session_unsubscribe(mqtt_session_t *session, int64_t handle)
    {
        if (!session)
            return -1;
        auto cpp_session = reinterpret_cast<mqtt::Session *>(session);
        return cpp_session->unsubscribe(handle) ? 0 : -1;
    }

    int64_t mqtt_session_publish(mqtt_session_t *session,
                                 const char *topic,
                                 const uint8_t *payload,
                                 size_t length,
                                 mqtt_qos_t qos,
                                 int retain)
    {
        if (!session)
            return -1;
        auto cpp_session = reinterpret_cast<mqtt::Session *>(session);
        return cpp_session->publish(topic, payload, length,
                                    static_cast<mqtt::Message::QoS>(qos),
                                    retain != 0);
    }

    // Message accessors
    const char *mqtt_message_get_topic(const mqtt_message_t *message)
    {
        if (!message)
            return nullptr;
        auto cpp_message = reinterpret_cast<const mqtt::Message *>(message);
        return cpp_message->getTopic();
    }

    const uint8_t *mqtt_message_get_payload(const mqtt_message_t *message)
    {
        if (!message)
            return nullptr;
        auto cpp_message = reinterpret_cast<const mqtt::Message *>(message);
        return cpp_message->getPayload();
    }

    size_t mqtt_message_get_payload_len(const mqtt_message_t *message)
    {
        if (!message)
            return 0;
        auto cpp_message = reinterpret_cast<const mqtt::Message *>(message);
        return cpp_message->getPayloadLength();
    }

    mqtt_qos_t mqtt_message_get_qos(const mqtt_message_t *message)
    {
        if (!message)
            return MQTT_QOS_AT_MOST_ONCE;
        auto cpp_message = reinterpret_cast<const mqtt::Message *>(message);
        return static_cast<mqtt_qos_t>(cpp_message->getQoS());
    }

    int mqtt_message_is_retained(const mqtt_message_t *message)
    {
        if (!message)
            return 0;
        auto cpp_message = reinterpret_cast<const mqtt::Message *>(message);
        return cpp_message->isRetained() ? 1 : 0;
    }

    // Configuration
    int mqtt_session_set_broker(mqtt_session_t *session,
                                const char *url,
                                uint16_t port)
    {
        if (!session)
            return -1;
        auto cpp_session = reinterpret_cast<mqtt::Session *>(session);
        cpp_session->getConfig().setBroker(url, port);
        return 0;
    }

    int mqtt_session_set_credentials(mqtt_session_t *session,
                                     const char *username,
                                     const char *password)
    {
        if (!session)
            return -1;
        auto cpp_session = reinterpret_cast<mqtt::Session *>(session);
        cpp_session->getConfig().setCredentials(username, password);
        return 0;
    }
}