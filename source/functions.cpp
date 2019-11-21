#include "functions.h"

#include <iostream>
#include <tuple>
#include <string>
#include <fstream>
#include <vector>

#include <glad/glad.h>
#include <GLFW/glfw3.h>

GLuint loadBMP(const std::string_view imagepath)
{
    std::ifstream file(imagepath.data(), std::ios::binary);

    if (!file)
    {
        std::cout << "File " << imagepath << " could not be opened\n";
        return 0;
    }

    char header[54];
    if (file.readsome(&header[0], std::size(header)) != 54)
    {
        std::cout << "Not a correct BMP file\n";
        return 0;
    }

    if (header[0] != 'B' || header[1] != 'M')
    {
        std::cout << "Not a correct BMP file\n";
        return 0;
    }
    if (*reinterpret_cast<int *>(&header[0x1E]) != 0)
    {
        std::cout << "Not a correct BMP file\n";
        return 0;
    }
    if (*reinterpret_cast<int *>(&header[0x1C]) != 24)
    {
        std::cout << "Not a correct BMP file\n";
        return 0;
    }

    int data_pos = *reinterpret_cast<int *>(&header[0x0A]);
    int image_size = *reinterpret_cast<int *>(&header[0x22]);
    int width = *reinterpret_cast<int *>(&header[0x12]);
    int height = *reinterpret_cast<int *>(&header[0x16]);

    if (image_size == 0)
        image_size = width * height * 3;
    if (data_pos == 0)
        data_pos = 54;

    std::vector<char> data(image_size);
    file.read(data.data(), image_size);
    file.close();

    GLuint texture;
    glGenTextures(1, &texture);
    glBindTexture(GL_TEXTURE_2D, texture);
    glTexImage2D(GL_TEXTURE_2D, 0, GL_RGB, width, height, 0, GL_BGR, GL_UNSIGNED_BYTE, data.data());

    glTexParameteri(GL_TEXTURE_2D, GL_TEXTURE_WRAP_S, GL_REPEAT);
    glTexParameteri(GL_TEXTURE_2D, GL_TEXTURE_WRAP_T, GL_REPEAT);
    glTexParameteri(GL_TEXTURE_2D, GL_TEXTURE_MAG_FILTER, GL_LINEAR);
    glTexParameteri(GL_TEXTURE_2D, GL_TEXTURE_MIN_FILTER, GL_LINEAR_MIPMAP_LINEAR);
    glGenerateMipmap(GL_TEXTURE_2D);

    return texture;
}

static GLuint compile_shader_object(const std::string_view shader_path, GLenum type)
{

    std::ifstream shader_stream(shader_path.data(), std::ios::in | std::ios::binary | std::ios::ate);
    std::string code;
    if (shader_stream.is_open())
    {
        const auto size = shader_stream.tellg();
        code.resize(size, '\0');
        shader_stream.seekg(0);
        shader_stream.read(&code[0], size);
    }
    else
    {
        std::cout << "Impossible to open" << shader_path << ".\n";
        return 0;
    }

    std::cout << "Compiling shader: " << shader_path << '\n';
    GLuint shaderID = glCreateShader(type);
    const auto cstr_code = code.c_str();
    glShaderSource(shaderID, 1, &cstr_code, NULL);
    glCompileShader(shaderID);

    auto result = GL_FALSE;
    glGetShaderiv(shaderID, GL_COMPILE_STATUS, &result);
    int infolog_length;
    glGetShaderiv(shaderID, GL_INFO_LOG_LENGTH, &infolog_length);
    if (infolog_length > 0)
    {
        std::string message(infolog_length, '\0');
        glGetShaderInfoLog(shaderID, infolog_length, nullptr, &message[0]);
        std::cout << message << '\n';
    }
    return shaderID;
}

GLuint load_shaders(const std::string_view vertex_shader_path, const std::string_view fragment_shader_path)
{
    const auto vertex_shaderID = compile_shader_object(vertex_shader_path, GL_VERTEX_SHADER);
    const auto fragment_shaderID = compile_shader_object(fragment_shader_path, GL_FRAGMENT_SHADER);

    std::cout << "Linking program\n";
    GLuint programID = glCreateProgram();
    glAttachShader(programID, vertex_shaderID);
    glAttachShader(programID, fragment_shaderID);
    glLinkProgram(programID);

    auto res = GL_FALSE;
    glGetProgramiv(programID, GL_LINK_STATUS, &res);
    int infolog_length;
    glGetProgramiv(programID, GL_INFO_LOG_LENGTH, &infolog_length);
    if (infolog_length > 0)
    {
        std::string message(infolog_length, '\0');
        glGetShaderInfoLog(programID, infolog_length, nullptr, &message[0]);
        std::cout << message << '\n';
    }

    glDetachShader(programID, vertex_shaderID);
    glDetachShader(programID, fragment_shaderID);
    glDeleteShader(vertex_shaderID);
    glDeleteShader(fragment_shaderID);
    return programID;
}

bool loadOBJ(
    const std::string_view path,
    std::vector<glm::vec3> &out_vertices,
    std::vector<glm::vec2> &out_uvs,
    std::vector<glm::vec3> &out_normals)
{

    std::cout << "Loading OBJ file" << path << "...\n";

    std::vector<unsigned int> vertex_ind, uv_ind, normal_ind;
    std::vector<glm::vec3> temp_vertices;
    std::vector<glm::vec2> temp_uvs;
    std::vector<glm::vec3> temp_normals;

    std::ifstream file(path.data(), std::ios::in);
    if (!file.is_open())
    {
        std::cout << "Impossible to open the file !";
        return false;
    }

    std::string word;
    while (file >> word)
    {
        if (word == "v")
        {
            glm::vec3 vertex;
            file >> vertex.x >> vertex.y >> vertex.z;
            temp_vertices.push_back(vertex);
        }
        else if (word == "vt")
        {
            glm::vec2 uv;
            file >> uv.x >> uv.y;
            uv.y = -uv.y;
            temp_uvs.push_back(uv);
        }
        else if (word == "vn")
        {
            glm::vec3 normal;
            file >> normal.x >> normal.y >> normal.z;
            temp_normals.push_back(normal);
        }
        else if (word == "f")
        {
            std::string str;
            for (int i = 0; i < 3; ++i)
            {
                std::getline(file >> std::ws, str, '/');
                vertex_ind.push_back(std::stoi(str));

                std::getline(file >> std::ws, str, '/');
                uv_ind.push_back(std::stoi(str));

                file >> std::ws >> str;
                normal_ind.push_back(std::stoi(str));
            }
        }
        else
        {
            file.ignore(std::numeric_limits<std::streamsize>::max(), '\n');
        }
    }

    for (unsigned int i = 0; i < vertex_ind.size(); ++i)
    {
        const glm::vec3 &vertex = temp_vertices[vertex_ind[i] - 1];
        const glm::vec2 &uv = temp_uvs[uv_ind[i] - 1];
        const glm::vec3 &normal = temp_normals[normal_ind[i] - 1];

        out_vertices.push_back(vertex);
        out_uvs.push_back(uv);
        out_normals.push_back(normal);
    }

    file.close();
    return true;
}

GLuint loadDDS(const std::string_view imagepath)
{

    std::cout << "Loading OBJ file" << imagepath << "...\n";

    std::ifstream file(imagepath.data(), std::ios::in | std::ios::binary);
    if (!file.is_open())
    {
        std::cout << "Impossible to open the file !";
        return 0;
    }
    std::string filecode(4, '\0');
    file.read(&filecode[0], filecode.size());
    if (filecode != "DDS ")
    {
        return 0;
    }

    char header[124];
    file.read(header, sizeof(header));

    unsigned int height = *reinterpret_cast<unsigned int *>(&header[8]);
    unsigned int width = *reinterpret_cast<unsigned int *>(&header[12]);
    unsigned int linear_size = *reinterpret_cast<unsigned int *>(&header[16]);
    unsigned int mipmap_count = *reinterpret_cast<unsigned int *>(&header[24]);
    unsigned int fourcc = *reinterpret_cast<unsigned int *>(&header[80]);

    const auto bufsize = mipmap_count > 1 ? linear_size * 2 : linear_size;
    std::string buffer(bufsize, '\0');
    file.read(&buffer[0], bufsize);
    file.close();

    constexpr unsigned int fourcc_DXT1 = 0x31545844; // Equivalent to "DXT1" in ASCII
    constexpr unsigned int fourcc_DXT3 = 0x33545844; // Equivalent to "DXT3" in ASCII
    constexpr unsigned int fourcc_DXT5 = 0x35545844; // Equivalent to "DXT5" in ASCII

    unsigned int format;
    switch (fourcc)
    {
    case fourcc_DXT1:
        format = GL_COMPRESSED_RGBA_S3TC_DXT1_EXT;
        break;
    case fourcc_DXT3:
        format = GL_COMPRESSED_RGBA_S3TC_DXT3_EXT;
        break;
    case fourcc_DXT5:
        format = GL_COMPRESSED_RGBA_S3TC_DXT5_EXT;
        break;
    default:
        return 0;
    }

    GLuint textureID;
    glGenTextures(1, &textureID);

    glBindTexture(GL_TEXTURE_2D, textureID);
    glPixelStorei(GL_UNPACK_ALIGNMENT, 1);

    unsigned int blockSize = (format == GL_COMPRESSED_RGBA_S3TC_DXT1_EXT) ? 8 : 16;
    unsigned int offset = 0;

    for (unsigned int level = 0; level < mipmap_count && (width || height); ++level)
    {
        const auto size = ((width + 3) / 4) * ((height + 3) / 4) * blockSize;
        glCompressedTexImage2D(GL_TEXTURE_2D, level, format, width, height,
                               0, size, &buffer[offset]);

        offset += size;
        width /= 2;
        height /= 2;

        if (width < 1)
            width = 1;
        if (height < 1)
            height = 1;
    }
    return textureID;
}
