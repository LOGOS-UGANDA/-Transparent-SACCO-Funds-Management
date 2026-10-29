# Transparent SACCO core module — universal interface (impl-only authoring).

cmake_minimum_required(VERSION 3.14)
project(SaccoCoreModulePlugin LANGUAGES CXX)

if(DEFINED ENV{LOGOS_MODULE_BUILDER_ROOT})
    include($ENV{LOGOS_MODULE_BUILDER_ROOT}/cmake/LogosModule.cmake)
elseif(EXISTS "${CMAKE_CURRENT_SOURCE_DIR}/cmake/LogosModule.cmake")
    include(cmake/LogosModule.cmake)
else()
    message(FATAL_ERROR "LogosModule.cmake not found")
endif()

file(READ "${CMAKE_CURRENT_SOURCE_DIR}/metadata.json" METADATA_JSON)
string(JSON MODULE_NAME GET ${METADATA_JSON} name)

logos_module(
    NAME ${MODULE_NAME}
    SOURCES
        src/sacco_core_impl.h
        src/sacco_core_impl.cpp
        src/sacco_ledger_client_ffi.h
)

find_program(CARGO_EXECUTABLE cargo REQUIRED)
find_library(SACCO_LIBCLANG NAMES clang libclang)
if(SACCO_LIBCLANG)
    get_filename_component(SACCO_LIBCLANG_DIR "${SACCO_LIBCLANG}" DIRECTORY)
    set(ENV{LIBCLANG_PATH} "${SACCO_LIBCLANG_DIR}")
endif()
if(DEFINED ENV{SACCO_PROGRAMS_ROOT})
    set(SACCO_PROGRAMS_ROOT "$ENV{SACCO_PROGRAMS_ROOT}")
else()
    set(SACCO_PROGRAMS_ROOT "${CMAKE_CURRENT_SOURCE_DIR}/../../programs")
endif()
set(SACCO_RUST_MANIFEST "${SACCO_PROGRAMS_ROOT}/sacco_ledger/ffi/Cargo.toml")
set(SACCO_RUST_TARGET_DIR "${CMAKE_CURRENT_BINARY_DIR}/sacco-rust-target")
set(SACCO_RUST_ARCHIVE "${SACCO_RUST_TARGET_DIR}/release/libsacco_ledger_ffi.a")
add_custom_command(
    OUTPUT "${SACCO_RUST_ARCHIVE}"
    COMMAND "${CMAKE_COMMAND}" -E env CARGO_BUILD_JOBS=2 "${CARGO_EXECUTABLE}"
        build --release --lib --offline --locked
        --manifest-path "${SACCO_RUST_MANIFEST}"
        --target-dir "${SACCO_RUST_TARGET_DIR}"
    WORKING_DIRECTORY "${SACCO_PROGRAMS_ROOT}/sacco_ledger/ffi"
    DEPENDS
        "${SACCO_PROGRAMS_ROOT}/sacco_ledger/ffi/Cargo.lock"
        "${SACCO_PROGRAMS_ROOT}/sacco_ledger/ffi/src/lib.rs"
        "${SACCO_PROGRAMS_ROOT}/sacco_ledger/ffi/.cargo/config.toml"
        "${SACCO_PROGRAMS_ROOT}/sacco_ledger/idl/src/lib.rs"
    COMMENT "Building the Rust sacco_ledger IDL/client bridge"
    VERBATIM
)
add_custom_target(sacco_ledger_client_ffi DEPENDS "${SACCO_RUST_ARCHIVE}")
add_dependencies(sacco_core_module_plugin sacco_ledger_client_ffi)
target_include_directories(sacco_core_module_plugin PRIVATE
    "${CMAKE_CURRENT_SOURCE_DIR}/src"
)
target_link_libraries(sacco_core_module_plugin PRIVATE "${SACCO_RUST_ARCHIVE}" pthread dl)
