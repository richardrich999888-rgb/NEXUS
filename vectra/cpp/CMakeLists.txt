cmake_minimum_required(VERSION 3.16)
project(vectra VERSION 0.1.0 LANGUAGES CXX)

set(CMAKE_CXX_STANDARD 20)
set(CMAKE_CXX_STANDARD_REQUIRED ON)
set(CMAKE_CXX_EXTENSIONS OFF)

# Compiler warnings
if(CMAKE_CXX_COMPILER_ID MATCHES "GNU|Clang")
    add_compile_options(-Wall -Wextra -Wpedantic -Werror)
endif()

# Library
add_library(vectra
    src/types.cpp
    src/decompose.cpp
    src/fee.cpp
    src/nsge.cpp
    src/ebta.cpp
    src/artifact.cpp
    src/integrity.cpp
    src/encode.cpp
    src/decode.cpp
)

target_include_directories(vectra PUBLIC
    $<BUILD_INTERFACE:${CMAKE_CURRENT_SOURCE_DIR}/include>
    $<INSTALL_INTERFACE:include>
)

# OpenSSL for SHA-256
find_package(OpenSSL REQUIRED)
target_link_libraries(vectra PRIVATE OpenSSL::Crypto)

# Tests
enable_testing()
find_package(GTest QUIET)

if(GTest_FOUND)
    add_executable(vectra_tests
        tests/test_vectra.cpp
    )
    target_link_libraries(vectra_tests vectra GTest::gtest_main)
    add_test(NAME vectra_tests COMMAND vectra_tests)
endif()

# Install
install(TARGETS vectra
    LIBRARY DESTINATION lib
    ARCHIVE DESTINATION lib
)
install(DIRECTORY include/ DESTINATION include)
