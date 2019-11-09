cmake_minimum_required(VERSION 3.8)

project(gt_test 
        VERSION 0.1.0 
        LANGUAGES CXX)


add_subdirectory(glad)

file(GLOB_RECURSE sources CONFIGURE_DEPENDS
    source/*.cpp            source/*.h
    )

find_package(glfw3 3.3 REQUIRED)
find_package(OpenGL REQUIRED)

add_executable(gl_test ${sources})

target_compile_features(gl_test 
                        PRIVATE 
                        cxx_std_17)

target_compile_options(gl_test
                        PRIVATE 
                        -Wall -Wextra -pedantic)

target_link_libraries(gl_test 
                        PRIVATE 
                        glfw ${CMAKE_DL_LIBS} glad OpenGL::OpenGL)                                
                                            