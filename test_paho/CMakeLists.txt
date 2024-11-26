cmake_minimum_required(VERSION 3.10)
project(paho-test CXX)

# Add parent cmake directory to module path
list(APPEND CMAKE_MODULE_PATH "${CMAKE_CURRENT_SOURCE_DIR}/../cmake")

# Find Paho MQTT C
find_package(PahoMQTTC REQUIRED)

# Create test executable
add_executable(paho_test test.cpp)

# Link against Paho MQTT C
target_link_libraries(paho_test PRIVATE ${PahoMQTTC_LIBRARIES})
target_include_directories(paho_test PRIVATE ${PahoMQTTC_INCLUDE_DIRS})

# Set C++ standard
set_target_properties(paho_test PROPERTIES
    CXX_STANDARD 17
    CXX_STANDARD_REQUIRED ON
)