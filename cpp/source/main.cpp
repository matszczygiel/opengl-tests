#include <iostream>
#include <tuple>
#include <string>
#include <fstream>
#include <vector>
#include <chrono>

#include <glad/glad.h>
#include <GLFW/glfw3.h>
#include <glm/glm.hpp>
#include <glm/gtc/matrix_transform.hpp>
#include <glm/gtc/type_ptr.hpp>
#include <stb/stb_image.h>

#include "functions.h"

std::tuple<glm::mat4, glm::mat4> matrices_from_input(GLFWwindow *window)
{
    static auto last_time = glfwGetTime();
    const auto current_time = glfwGetTime();
    const auto delta_time = static_cast<float>(current_time - last_time);
    last_time = current_time;

    static auto horizontal_angle = 3.14f;
    static auto vertical_angle = 0.0f;
    static glm::vec3 position = glm::vec3(0.0f, 0.0f, 5.0f);
    static float fov = 45.0f;

    constexpr auto mouse_speed = 0.002f;

    double xpos, ypos;
    glfwGetCursorPos(window, &xpos, &ypos);
    static double last_xpos = xpos;
    static double last_ypos = ypos;

    horizontal_angle += mouse_speed * static_cast<float>(last_xpos - xpos);
    vertical_angle += mouse_speed * static_cast<float>(last_ypos - ypos);
    last_xpos = xpos;
    last_ypos = ypos;

    const glm::vec3 direction(
        glm::cos(vertical_angle) * glm::sin(horizontal_angle),
        glm::sin(vertical_angle),
        glm::cos(vertical_angle) * glm::cos(horizontal_angle));

    const glm::vec3 right = glm::vec3(
        glm::sin(horizontal_angle - 3.14f / 2.0f),
        0,
        glm::cos(horizontal_angle - 3.14f / 2.0f));

    const glm::vec3 up = glm::cross(right, direction);

    constexpr auto speed = 3.0f;

    if (glfwGetKey(window, GLFW_KEY_UP) == GLFW_PRESS)
    {
        position += direction * delta_time * speed;
    }
    if (glfwGetKey(window, GLFW_KEY_DOWN) == GLFW_PRESS)
    {
        position -= direction * delta_time * speed;
    }
    if (glfwGetKey(window, GLFW_KEY_RIGHT) == GLFW_PRESS)
    {
        position += right * delta_time * speed;
    }
    if (glfwGetKey(window, GLFW_KEY_LEFT) == GLFW_PRESS)
    {
        position -= right * delta_time * speed;
    }

    constexpr auto fov_speed = 1.05f;

    if (glfwGetKey(window, GLFW_KEY_W) == GLFW_PRESS)
    {
        fov /= fov_speed;
        if (fov < 15.0f)
            fov = 15.0f;
    }
    if (glfwGetKey(window, GLFW_KEY_S) == GLFW_PRESS)
    {
        fov *= fov_speed;
        if (fov > 100.0f)
            fov = 100.0f;
    }
    if (glfwGetKey(window, GLFW_KEY_R) == GLFW_PRESS)
    {
        horizontal_angle = 3.14f;
        vertical_angle = 0.0f;
        position = glm::vec3(0.0f, 0.0f, 5.0f);
        fov = 45.0f;
    }

    int width, height;
    glfwGetWindowSize(window, &width, &height);
    const auto proj = glm::perspective(glm::radians(fov),
                                       static_cast<float>(width) / static_cast<float>(height),
                                       0.1f,
                                       100.0f);

    const auto view = glm::lookAt(
        position,
        position + direction,
        up);

    return std::make_tuple(view, proj);
}

void GLAPIENTRY
msg_callback(GLenum source,
             GLenum type,
             GLuint id,
             GLenum severity,
             GLsizei length,
             const GLchar *message,
             const void *userParam)
{
    std::cout << "GL CALLBACK: ";
    if (type == GL_DEBUG_TYPE_ERROR)
        std::cout << "** GL ERROR **";

    std::cout << " type = " << type << ", severity = " << severity << ", message =  " << message << "\n";
}

int main()
{

    if (!glfwInit())
        return -1;

    glfwSetErrorCallback([](int error, const char *description) {
        std::cout << "glfw Error " << error << ": " << description << '\n';
    });

    constexpr int window_width = 1280;
    constexpr int window_height = 960;

    GLFWwindow *window;
    glfwWindowHint(GLFW_SAMPLES, 4);
    glfwWindowHint(GLFW_CONTEXT_VERSION_MAJOR, 4);
    glfwWindowHint(GLFW_CONTEXT_VERSION_MINOR, 3);
    glfwWindowHint(GLFW_OPENGL_PROFILE, GLFW_OPENGL_CORE_PROFILE);
    window = glfwCreateWindow(window_width, window_height, "Hello World", nullptr, nullptr);
    if (!window)
    {
        glfwTerminate();
        return -1;
    }

    glfwMakeContextCurrent(window);
    gladLoadGLLoader((GLADloadproc)glfwGetProcAddress);

    std::cout << "OpenGL Info\n";
    std::cout << "  Vendor:   " << glGetString(GL_VENDOR) << '\n';
    std::cout << "  Renderer: " << glGetString(GL_RENDERER) << '\n';
    std::cout << "  Version:  " << glGetString(GL_VERSION) << '\n';

    glfwSwapInterval(1);
    glfwSetInputMode(window, GLFW_CURSOR, GLFW_CURSOR_DISABLED);
    glfwSetInputMode(window, GLFW_STICKY_KEYS, GLFW_TRUE);

    glClearColor(0.0f, 0.0f, 0.4f, 0.0f);

    glEnable(GL_DEBUG_OUTPUT);
    glDebugMessageCallback(msg_callback, 0);

    glEnable(GL_DEPTH_TEST);
    glDepthFunc(GL_LESS);
    glEnable(GL_CULL_FACE);

    glEnable(GL_BLEND);
    glBlendFunc(GL_SRC_ALPHA, GL_ONE_MINUS_SRC_ALPHA);

    const auto texture_id = loadDDS("../resources/diffuse.DDS");
    const auto normal_texture_id = loadBMP("../resources/normal.bmp");
    const auto specular_texture_id = loadBMP("../resources/specular.DDS");

    GLuint vertex_array;
    glGenVertexArrays(1, &vertex_array);
    glBindVertexArray(vertex_array);

    std::vector<glm::vec3> vertices;
    std::vector<glm::vec2> uvs;
    std::vector<glm::vec3> normals;
    std::vector<unsigned int> indices;
    bool res = loadOBJ("../resources/cylinder.obj", vertices, uvs, normals, indices);
    assert(res);

    std::vector<glm::vec3> tangents;
    std::vector<glm::vec3> bitangents;
    compute_tangent_basis(vertices, uvs, normals, indices, tangents, bitangents);

    GLuint vertex_buffer;
    glGenBuffers(1, &vertex_buffer);
    glBindBuffer(GL_ARRAY_BUFFER, vertex_buffer);
    glBufferData(GL_ARRAY_BUFFER, vertices.size() * sizeof(glm::vec3), vertices.data(), GL_STATIC_DRAW);

    GLuint uv_buffer;
    glGenBuffers(1, &uv_buffer);
    glBindBuffer(GL_ARRAY_BUFFER, uv_buffer);
    glBufferData(GL_ARRAY_BUFFER, uvs.size() * sizeof(glm::vec2), uvs.data(), GL_STATIC_DRAW);

    GLuint normal_buffer;
    glGenBuffers(1, &normal_buffer);
    glBindBuffer(GL_ARRAY_BUFFER, normal_buffer);
    glBufferData(GL_ARRAY_BUFFER, normals.size() * sizeof(glm::vec3), normals.data(), GL_STATIC_DRAW);

    GLuint tangent_buffer;
    glGenBuffers(1, &tangent_buffer);
    glBindBuffer(GL_ARRAY_BUFFER, tangent_buffer);
    glBufferData(GL_ARRAY_BUFFER, tangents.size() * sizeof(glm::vec3), tangents.data(), GL_STATIC_DRAW);

    GLuint bitangent_buffer;
    glGenBuffers(1, &bitangent_buffer);
    glBindBuffer(GL_ARRAY_BUFFER, bitangent_buffer);
    glBufferData(GL_ARRAY_BUFFER, bitangents.size() * sizeof(glm::vec3), bitangents.data(), GL_STATIC_DRAW);

    GLuint index_buffer;
    glGenBuffers(1, &index_buffer);
    glBindBuffer(GL_ELEMENT_ARRAY_BUFFER, index_buffer);
    glBufferData(GL_ELEMENT_ARRAY_BUFFER, indices.size() * sizeof(unsigned int), indices.data(), GL_STATIC_DRAW);

    const auto program = load_shaders("../shaders/shader.vert", "../shaders/shader.frag");
    glUseProgram(program);
    const auto mvp_uniform = glGetUniformLocation(program, "MVP");
    const auto mv3_uniform = glGetUniformLocation(program, "MV3");
    const auto v_uniform = glGetUniformLocation(program, "V");
    const auto m_uniform = glGetUniformLocation(program, "M");
    const auto texture_samp_uniform = glGetUniformLocation(program, "texture_samp");
    const auto normal_texture_samp_uniform = glGetUniformLocation(program, "normal_texture_samp");
    const auto specular_texture_samp_uniform = glGetUniformLocation(program, "specular_texture_samp");
    const auto light_pos_uniform = glGetUniformLocation(program, "light_pos_worldspace");
    const auto light_color_uniform = glGetUniformLocation(program, "light_color");
    const auto light_power_uniform = glGetUniformLocation(program, "light_power");

    GLuint cube_texture;
    glGenTextures(1, &cube_texture);
    glBindTexture(GL_TEXTURE_CUBE_MAP, cube_texture);

    constexpr std::array<std::string_view, 6> texture_faces = {"../resources/cubemap/px.png",
                                                               "../resources/cubemap/nx.png",
                                                               "../resources/cubemap/py.png",
                                                               "../resources/cubemap/ny.png",
                                                               "../resources/cubemap/pz.png",
                                                               "../resources/cubemap/nz.png"};

    //stbi_set_flip_vertically_on_load(1);
    int width, height, nrChannels;
    unsigned char *data;
    for (GLuint i = 0; i < texture_faces.size(); i++)
    {
        data = stbi_load(texture_faces[i].data(), &width, &height, &nrChannels, STBI_rgb);
        if (data)
        {
            glTexImage2D(GL_TEXTURE_CUBE_MAP_POSITIVE_X + i,
                         0, GL_RGB8, width, height, 0, GL_RGB, GL_UNSIGNED_BYTE, data);
        }
        else
        {
            std::cout << "Cubemap texture failed to load at path: " << texture_faces[i] << '\n';
        }
        stbi_image_free(data);
    }

    glTexParameteri(GL_TEXTURE_CUBE_MAP, GL_TEXTURE_MIN_FILTER, GL_LINEAR);
    glTexParameteri(GL_TEXTURE_CUBE_MAP, GL_TEXTURE_MAG_FILTER, GL_LINEAR);
    glTexParameteri(GL_TEXTURE_CUBE_MAP, GL_TEXTURE_WRAP_S, GL_CLAMP_TO_EDGE);
    glTexParameteri(GL_TEXTURE_CUBE_MAP, GL_TEXTURE_WRAP_T, GL_CLAMP_TO_EDGE);
    glTexParameteri(GL_TEXTURE_CUBE_MAP, GL_TEXTURE_WRAP_R, GL_CLAMP_TO_EDGE);

    float skybox_vertices[] = {
        // positions
        -1.0f, 1.0f, -1.0f,
        -1.0f, -1.0f, -1.0f,
        1.0f, -1.0f, -1.0f,
        1.0f, -1.0f, -1.0f,
        1.0f, 1.0f, -1.0f,
        -1.0f, 1.0f, -1.0f,

        -1.0f, -1.0f, 1.0f,
        -1.0f, -1.0f, -1.0f,
        -1.0f, 1.0f, -1.0f,
        -1.0f, 1.0f, -1.0f,
        -1.0f, 1.0f, 1.0f,
        -1.0f, -1.0f, 1.0f,

        1.0f, -1.0f, -1.0f,
        1.0f, -1.0f, 1.0f,
        1.0f, 1.0f, 1.0f,
        1.0f, 1.0f, 1.0f,
        1.0f, 1.0f, -1.0f,
        1.0f, -1.0f, -1.0f,

        -1.0f, -1.0f, 1.0f,
        -1.0f, 1.0f, 1.0f,
        1.0f, 1.0f, 1.0f,
        1.0f, 1.0f, 1.0f,
        1.0f, -1.0f, 1.0f,
        -1.0f, -1.0f, 1.0f,

        -1.0f, 1.0f, -1.0f,
        1.0f, 1.0f, -1.0f,
        1.0f, 1.0f, 1.0f,
        1.0f, 1.0f, 1.0f,
        -1.0f, 1.0f, 1.0f,
        -1.0f, 1.0f, -1.0f,

        -1.0f, -1.0f, -1.0f,
        -1.0f, -1.0f, 1.0f,
        1.0f, -1.0f, -1.0f,
        1.0f, -1.0f, -1.0f,
        -1.0f, -1.0f, 1.0f,
        1.0f, -1.0f, 1.0f};

    GLuint skybox_vertex_array;
    glGenVertexArrays(1, &skybox_vertex_array);
    glBindVertexArray(skybox_vertex_array);

    GLuint skybox_vertex_buffer;
    glGenBuffers(1, &skybox_vertex_buffer);
    glBindBuffer(GL_ARRAY_BUFFER, skybox_vertex_buffer);
    glBufferData(GL_ARRAY_BUFFER, sizeof(skybox_vertices), skybox_vertices, GL_STATIC_DRAW);

    const auto skybox_program = load_shaders("../shaders/skybox.vert", "../shaders/skybox.frag");

    const auto skybox_projection_uniform = glGetUniformLocation(skybox_program, "projection");
    const auto skybox_view_uniform = glGetUniformLocation(skybox_program, "view");
    const auto skybox_uniform = glGetUniformLocation(skybox_program, "skybox");

    auto light_position = glm::vec3(4.0, 4.0, 4.0);
    auto light_color = glm::vec3(1.0, 1.0, 1.0);
    auto light_power = 200.0f;

    auto start = std::chrono::system_clock::now();
    while (!glfwWindowShouldClose(window))
    {
        glClear(GL_COLOR_BUFFER_BIT | GL_DEPTH_BUFFER_BIT);

        const auto [view, projection] = matrices_from_input(window);
        const auto model = glm::mat4(1.0f);
        const auto mv = view * model;
        const auto mvp = projection * mv;
        const auto mv3 = glm::mat3(mv);

        glUseProgram(program);

        glUniformMatrix4fv(mvp_uniform, 1, GL_FALSE, glm::value_ptr(mvp));
        glUniformMatrix4fv(v_uniform, 1, GL_FALSE, glm::value_ptr(view));
        glUniformMatrix4fv(m_uniform, 1, GL_FALSE, glm::value_ptr(model));
        glUniformMatrix3fv(mv3_uniform, 1, GL_FALSE, glm::value_ptr(mv3));

        glUniform3f(light_pos_uniform, light_position.x, light_position.y, light_position.z);
        glUniform3f(light_color_uniform, light_color.r, light_color.g, light_color.b);
        glUniform1f(light_power_uniform, light_power);

        glActiveTexture(GL_TEXTURE0);
        glBindTexture(GL_TEXTURE_2D, texture_id);
        glUniform1i(texture_samp_uniform, 0);

        glActiveTexture(GL_TEXTURE1);
        glBindTexture(GL_TEXTURE_2D, normal_texture_id);
        glUniform1i(normal_texture_samp_uniform, 1);

        glActiveTexture(GL_TEXTURE2);
        glBindTexture(GL_TEXTURE_2D, specular_texture_id);
        glUniform1i(specular_texture_samp_uniform, 2);

        glEnableVertexAttribArray(0);
        glBindBuffer(GL_ARRAY_BUFFER, vertex_buffer);
        glVertexAttribPointer(0, 3, GL_FLOAT, GL_FALSE, 0, (void *)0);

        glEnableVertexAttribArray(1);
        glBindBuffer(GL_ARRAY_BUFFER, uv_buffer);
        glVertexAttribPointer(1, 2, GL_FLOAT, GL_FALSE, 0, (void *)0);

        glEnableVertexAttribArray(2);
        glBindBuffer(GL_ARRAY_BUFFER, normal_buffer);
        glVertexAttribPointer(2, 3, GL_FLOAT, GL_FALSE, 0, (void *)0);

        glEnableVertexAttribArray(3);
        glBindBuffer(GL_ARRAY_BUFFER, tangent_buffer);
        glVertexAttribPointer(3, 3, GL_FLOAT, GL_FALSE, 0, (void *)0);

        glEnableVertexAttribArray(4);
        glBindBuffer(GL_ARRAY_BUFFER, bitangent_buffer);
        glVertexAttribPointer(4, 3, GL_FLOAT, GL_FALSE, 0, (void *)0);

        glBindBuffer(GL_ELEMENT_ARRAY_BUFFER, index_buffer);

        glDrawElements(GL_TRIANGLES, indices.size(), GL_UNSIGNED_INT, (void *)0);

        glDisableVertexAttribArray(0);
        glDisableVertexAttribArray(1);
        glDisableVertexAttribArray(2);
        glDisableVertexAttribArray(3);
        glDisableVertexAttribArray(4);

        glUseProgram(skybox_program);

        glBindVertexArray(skybox_vertex_array);
        glActiveTexture(GL_TEXTURE0);
        glBindTexture(GL_TEXTURE_CUBE_MAP, cube_texture);

        glUniform1i(skybox_uniform, 0);
        const auto skybox_view = glm::mat4(glm::mat3(view));
        glUniformMatrix4fv(skybox_view_uniform, 1, GL_FALSE, glm::value_ptr(skybox_view));
        glUniformMatrix4fv(skybox_projection_uniform, 1, GL_FALSE, glm::value_ptr(projection));

        glEnableVertexAttribArray(0);
        glBindBuffer(GL_ARRAY_BUFFER, skybox_vertex_buffer);
        glVertexAttribPointer(0, 3, GL_FLOAT, GL_FALSE, 0, (void *)0);

        glDepthFunc(GL_LEQUAL);
        glDrawArrays(GL_TRIANGLES, 0, 36);
        glDepthFunc(GL_LESS);

        glDisableVertexAttribArray(0);

        glBindVertexArray(vertex_array);

        glfwSwapBuffers(window);

        glfwPollEvents();

        if (glfwGetKey(window, GLFW_KEY_ESCAPE) == GLFW_PRESS)
            break;

        const auto end = std::chrono::system_clock::now();
        const auto delta_time = std::chrono::duration_cast<std::chrono::milliseconds>(end - start);
        start = end;
        std::cout << delta_time.count() << "ms\n";
    }

    glDeleteBuffers(1, &vertex_buffer);
    glDeleteBuffers(1, &uv_buffer);
    glDeleteBuffers(1, &normal_buffer);
    glDeleteBuffers(1, &tangent_buffer);
    glDeleteBuffers(1, &bitangent_buffer);
    glDeleteBuffers(1, &index_buffer);

    glDeleteTextures(1, &texture_id);
    glDeleteTextures(1, &normal_texture_id);

    glDeleteVertexArrays(1, &vertex_array);
    glDeleteProgram(program);

    glDeleteBuffers(1, &skybox_vertex_buffer);
    glDeleteVertexArrays(1, &skybox_vertex_array);
    glDeleteTextures(1, &cube_texture);
    glDeleteProgram(skybox_program);

    glfwTerminate();
}