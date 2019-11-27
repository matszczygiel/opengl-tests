cmake_minimum_required(VERSION 3.8)

project(stb 
        LANGUAGES CXX)
        
add_library(stb STATIC
                stb_image.cpp   include/stb/stb_image.h
            )


target_include_directories(stb 
            PUBLIC
            include)                 

target_compile_options(stb
                        PRIVATE 
                        -Wall -Wextra -pedantic)