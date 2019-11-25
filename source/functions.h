#pragma once

#include <string_view>
#include <vector>

#include <glad/glad.h>
#include <glm/glm.hpp>

GLuint loadBMP(const std::string_view imagepath);

GLuint load_shaders(const std::string_view vertex_shader_path, const std::string_view fragment_shader_path);

bool loadOBJ(
    const std::string_view path,
    std::vector<glm::vec3> &out_vertices,
    std::vector<glm::vec2> &out_uvs,
    std::vector<glm::vec3> &out_normals,
    std::vector<unsigned int>&out_indexes);

GLuint loadDDS(const std::string_view imagepath);
