cmake_minimum_required(VERSION 3.8)

project(glad 
        LANGUAGES C)
        
add_library(glad STATIC 
            src/glad.c
            include/glad/glad.h            
            include/KHR/khrplatform.h            
            )

target_include_directories(glad 
                            PUBLIC
                            include)         

target_compile_options(glad
                        PRIVATE 
                        -Wall -Wextra -pedantic)