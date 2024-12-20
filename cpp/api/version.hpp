#ifndef MQTT_API_VERSION_HPP_
#define MQTT_API_VERSION_HPP_

/* Auto-generated by build system */
#define MQTT_API_VERSION_CODE 1234
#define MQTT_API_VERSION_STRING "2.0.1.3"
#define MQTT_API_VERSION(a, b, c) (((a) << 16) + ((b) << 8) + (c))
#define MQTT_API_VERSION_MAJOR ((MQTT_API_VERSION_CODE & 0xff0000) >> 16)
#define MQTT_API_VERSION_MINOR ((MQTT_API_VERSION_CODE & 0x00ff00) >> 8)
#define MQTT_API_VERSION_RELEASE (MQTT_API_VERSION_CODE & 0x0000ff)
#define MQTT_API_PACKAGE_NAME "mqtt-api"
#define MQTT_API_PACKAGE_DATE "20160326"

#endif // MQTT_API_VERSION_HPP_