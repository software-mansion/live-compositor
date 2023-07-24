cmake_minimum_required(VERSION 3.15)

set(PROJECT_NAME rustium_cef_wrapper)

project(${PROJECT_NAME})

# TODO: Pass app name
set(HELPER_TARGET "rustium-demo_Helper")
set(HELPER_OUTPUT_NAME "rustium-demo Helper")
set(HELPER_SOURCES mac_helper.cc)

set(CEF_ROOT $ENV{CEF_ROOT})
set(_CEF_ROOT_EXPLICIT 1)
set(CMAKE_MODULE_PATH ${CMAKE_MODULE_PATH} ${CEF_ROOT}/cmake)

include("cef_macros")
include("cef_variables")

include_directories(${CEF_ROOT}/libcef_dll)
add_subdirectory(${CEF_ROOT} libcef_dll)

install(TARGETS libcef_dll_wrapper DESTINATION .)


foreach (_suffix_list ${CEF_HELPER_APP_SUFFIXES})
    # Convert to a list and extract the suffix values.
    string(REPLACE ":" ";" _suffix_list ${_suffix_list})
    list(GET _suffix_list 0 _name_suffix)
    list(GET _suffix_list 1 _target_suffix)
    list(GET _suffix_list 2 _plist_suffix)

    # Define Helper target and output names.
    set(_helper_target "${HELPER_TARGET}${_target_suffix}")
    set(_helper_output_name "${HELPER_OUTPUT_NAME}${_name_suffix}")

    # Create Helper-specific variants of the helper-Info.plist file. Do this
    # manually because the configure_file command (which is executed as part of
    # MACOSX_BUNDLE_INFO_PLIST) uses global env variables and would insert the
    # wrong values with multiple targets.
    set(_helper_info_plist "${CMAKE_CURRENT_BINARY_DIR}/helper-Info${_target_suffix}.plist")
    file(READ "${CMAKE_CURRENT_SOURCE_DIR}/resources/helper-Info.plist" _plist_contents)
    string(REPLACE "\${EXECUTABLE_NAME}" "${_helper_output_name}" _plist_contents ${_plist_contents})
    string(REPLACE "\${PRODUCT_NAME}" "${_helper_output_name}" _plist_contents ${_plist_contents})
    string(REPLACE "\${BUNDLE_ID_SUFFIX}" "${_plist_suffix}" _plist_contents ${_plist_contents})
    file(WRITE ${_helper_info_plist} ${_plist_contents})

    # Create Helper executable target.
    add_executable(${_helper_target} MACOSX_BUNDLE ${HELPER_SOURCES})
    SET_EXECUTABLE_TARGET_PROPERTIES(${_helper_target})

    target_include_directories(${_helper_target} PRIVATE ${CEF_ROOT})
    add_dependencies(${_helper_target} libcef_dll_wrapper)
    target_link_libraries(${_helper_target} libcef_dll_wrapper ${CEF_STANDARD_LIBS})
    set_target_properties(${_helper_target} PROPERTIES
            MACOSX_BUNDLE_INFO_PLIST ${_helper_info_plist}
            OUTPUT_NAME ${_helper_output_name}
            )

    # Copy the Helper app bundle into the Frameworks directory.
    add_custom_command(
            TARGET ${_helper_target}
            POST_BUILD
            COMMAND ${CMAKE_COMMAND} -E copy_directory
            "${CMAKE_CURRENT_BINARY_DIR}/${_helper_output_name}.app"
            "${OUT_DIR}/${_helper_output_name}.app"
            VERBATIM
    )

#    COPY_MAC_RESOURCES()
endforeach ()

