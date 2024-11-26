#include <MQTTClient.h>
#include <iostream>

void printVersion()
{
    MQTTClient_nameValue *version = MQTTClient_getVersionInfo();
    if (version)
    {
        for (int i = 0; version[i].name; ++i)
        {
            std::cout << version[i].name << ": " << version[i].value << std::endl;
        }
    }
    else
    {
        std::cout << "Unable to get version information" << std::endl;
    }
}

int main()
{
    printVersion();

    MQTTClient client;
    int rc = MQTTClient_create(&client,
                               "tcp://localhost:1883",
                               "TestClient",
                               MQTTCLIENT_PERSISTENCE_NONE,
                               nullptr);

    if (rc == MQTTCLIENT_SUCCESS)
    {
        std::cout << "Successfully created MQTT client\n";
        MQTTClient_destroy(&client);
        return 0;
    }
    else
    {
        std::cerr << "Failed to create MQTT client (rc=" << rc << ")\n";
        return 1;
    }
}