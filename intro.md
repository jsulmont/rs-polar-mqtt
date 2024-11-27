# Flattening a Legacy C++ API for Rust Integration

*Note*: **PolarMqtt** is a hypothetical library introduced for the purpose of this demonstration. It is based on [Eclipse paho.mqtt.cpp](https://github.com/eclipse-paho/paho.mqtt.cpp). 


When dealing with a legacy C++ API like **PolarMqtt**—which includes complex features such as pure virtual callbacks—integrating it directly into Rust can be challenging. Tools like `bindgen` often struggle with C++ code, especially when dealing with advanced features, making the process complex and fragile. To overcome these issues, we can **flatten** the C++ API into a C interface, facilitating easier integration with Rust.


## The Challenge

- **Legacy C++ API**: PolarMqtt is written in C++ and uses features like inheritance and pure virtual methods (abstract classes) for callbacks.
- **Complex Callbacks**: The API relies on pure virtual callbacks, which are difficult to represent in Rust.
- **Fragile Bindings**: Using `bindgen` to generate Rust bindings for C++ code is error-prone and often fails with complex C++ features.

## The Solution: Flattening the API

**Flattening** involves creating a C-compatible layer over the C++ API, exposing its functionality through a set of C functions and opaque pointers. This approach simplifies the interface, making it easier for Rust (and other languages) to interact with the API.

### Steps to Flatten the API

1. **Create Opaque Handles**: Define opaque pointer types in C to represent C++ objects.

 ```c
   typedef struct mqtt_session_t* mqtt_session_handle_t;
   int mqtt_session_start(mqtt_session_handle_t session);
int mqtt_publish(mqtt_session_handle_t session, const char* topic, const uint8_t* payload, size_t length, mqtt_qos_t qos, int retain);
```

2. 	**Expose C Functions**: Provide C functions that wrap the C++ methods, using the opaque handles.

 ```c
typedef void (*mqtt_message_callback_t)(const mqtt_message_data_t* message, void* user_context);
```

3. **Handle Callbacks via Function Pointers**: Define C-style function pointers for callbacks and manage them within the C layer.

 ```c
 typedef void (*mqtt_message_callback_t)(const mqtt_message_data_t* message, void* user_context);
 ```
 
4. **Implement Callback Mechanism**: In the C++ code, when a callback is needed, invoke the C function pointer stored in the C layer.

 ```cpp
 void onMessage(const mqtt::Message& message) override {
      if (message_cb_) {
         message_cb_(&msg_data, user_context_);
      }
}
```

5. **Generate Rust Bindings Using bindgen**: Use bindgen to generate Rust FFI bindings from the C headers.

 ```rust
 // Automatically generated bindings
include!(concat!(env!("OUT_DIR"), "/bindings.rs"));
```

### Advantages of Flattening

-	Simplifies Integration: C interfaces are easier to bind to Rust using FFI.
-	Avoids C++ Complexities: Bypasses issues with templates, exceptions, and pure virtual methods.
-	Stable ABI: C has a stable ABI, ensuring compatibility across different compilers and platforms.
-	Reusable in Other Languages: Other languages that can interface with C can also use the flattened API.


## Handling Pure Virtual Callbacks

Pure virtual methods in C++ represent abstract methods that must be implemented by derived classes. To handle these in the flattened C API:


1. **Define C Callback Types**: Create function pointer types in C that correspond to the callbacks.

 ```c
 typedef void (*mqtt_state_callback_t)(mqtt_session_state_t new_state, void* user_context);
 ```
 
2. **Store Callbacks in Context Structures**: When creating a session, pass the callbacks and user context, storing them in a struct.

 ```c
 typedef struct mqtt_session_t {
       mqtt_state_callback_t state_cb;
       void* user_context;
} mqtt_session_t;
```

3. **Implement C++ Classes to Invoke C Callbacks**: In the C++ layer, implement the pure virtual methods to call the stored C callbacks.

 ```c
 class SessionCallbackHandler : public mqtt::SessionHandler {
public:
      void onStateChange(mqtt::SessionState newState) override {
            if (state_cb_) {
               state_cb_(static_cast<mqtt_session_state_t>(newState), context_);
        }
      }
};