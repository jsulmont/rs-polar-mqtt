#ifndef MQTT_C_HPP
#define MQTT_C_HPP

#include <stdint.h>
#include <stddef.h>

#ifdef __cplusplus
extern "C"
{
#endif

    // Opaque types
    typedef struct mqtt_session_t mqtt_session_t;
    typedef struct mqtt_message_t mqtt_message_t;
    typedef struct mqtt_factory_t mqtt_factory_t;

    // Enums
    typedef enum
    {
        MQTT_QOS_AT_MOST_ONCE = 0,
        MQTT_QOS_AT_LEAST_ONCE = 1,
        MQTT_QOS_EXACTLY_ONCE = 2
    } mqtt_qos_t;

    typedef enum
    {
        MQTT_STATE_DISCONNECTED = 0,
        MQTT_STATE_CONNECTING = 1,
        MQTT_STATE_CONNECTED = 2,
        MQTT_STATE_RECONNECTING = 3
    } mqtt_state_t;

    // Callbacks
    typedef void (*mqtt_message_callback_t)(const mqtt_message_t *message, void *context);
    typedef void (*mqtt_state_callback_t)(mqtt_state_t state, void *context);

    // Factory functions
    mqtt_factory_t *mqtt_factory_get_instance(void);
    void mqtt_factory_destroy(mqtt_factory_t *factory);
    int mqtt_factory_initialize(const char *app_name, const char *app_version, int debug);

    // Session management
    mqtt_session_t *mqtt_session_create(mqtt_factory_t *factory,
                                        const char *client_id,
                                        mqtt_state_callback_t state_cb,
                                        void *state_context);
    void mqtt_session_destroy(mqtt_session_t *session);
    int mqtt_session_start(mqtt_session_t *session);
    int mqtt_session_stop(mqtt_session_t *session);
    mqtt_state_t mqtt_session_get_state(const mqtt_session_t *session);

    // Session operations
    int64_t mqtt_session_subscribe(mqtt_session_t *session,
                                   const char *topic,
                                   mqtt_qos_t qos,
                                   mqtt_message_callback_t msg_cb,
                                   void *msg_context);

    int mqtt_session_unsubscribe(mqtt_session_t *session, int64_t handle);

    int64_t mqtt_session_publish(mqtt_session_t *session,
                                 const char *topic,
                                 const uint8_t *payload,
                                 size_t length,
                                 mqtt_qos_t qos,
                                 int retain);

    // Message accessors
    const char *mqtt_message_get_topic(const mqtt_message_t *message);
    const uint8_t *mqtt_message_get_payload(const mqtt_message_t *message);
    size_t mqtt_message_get_payload_len(const mqtt_message_t *message);
    mqtt_qos_t mqtt_message_get_qos(const mqtt_message_t *message);
    int mqtt_message_is_retained(const mqtt_message_t *message);

    // Configuration
    int mqtt_session_set_broker(mqtt_session_t *session,
                                const char *url,
                                uint16_t port);

    int mqtt_session_set_credentials(mqtt_session_t *session,
                                     const char *username,
                                     const char *password);

#ifdef __cplusplus
}
#endif

#endif // MQTT_C_HPP