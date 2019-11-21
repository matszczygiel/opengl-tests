#include <iostream>
#include <tuple>
#include <string>
#include <fstream>
#include <vector>

#include <glad/glad.h>
#include <GLFW/glfw3.h>
#include <glm/glm.hpp>
#include <glm/gtc/matrix_transform.hpp>
#include <glm/gtc/type_ptr.hpp>

#include "functions.h"

std::tuple<glm::mat4, glm::mat4> matrices_from_input(GLFWwindow *window)
{
    static auto last_time = glfwGetTime();
    const auto current_time = glfwGetTime();
    const auto delta_time = static_cast<float>(current_time - last_time);
    last_time = current_time;

    static auto horizontal_angle = 3.14f;
    static auto vertical_angle = 0.0f;
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

    static glm::vec3 position = glm::vec3(0.0f, 0.0f, 5.0f);
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

    static float fov = 45.0f;
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

int main()
{

    if (!glfwInit())
        return -1;

    glfwSetErrorCallback([](int error, const char *description) {
        std::cout << "Error " << error << ": " << description << '\n';
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
    glEnable(GL_DEPTH_TEST);
    glDepthFunc(GL_LESS);
    glEnable(GL_CULL_FACE);

    const auto program = load_shaders("shaders/shader.vert", "shaders/shader.frag");
    const auto mvp_uniform = glGetUniformLocation(program, "MVP");
    const auto texture_samp = glGetUniformLocation(program, "texture_samp");

    const auto texture_id = loadDDS("resources/uvmap.DDS");

    GLuint vertex_array;
    glGenVertexArrays(1, &vertex_array);
    glBindVertexArray(vertex_array);

    std::vector<glm::vec3> vertices;
    std::vector<glm::vec2> uvs;
    std::vector<glm::vec3> normals;
    bool res = loadOBJ("resources/cube.obj", vertices, uvs, normals);

    GLuint vertex_buffer;
    glGenBuffers(1, &vertex_buffer);
    glBindBuffer(GL_ARRAY_BUFFER, vertex_buffer);
    glBufferData(GL_ARRAY_BUFFER, vertices.size() * sizeof(glm::vec3), vertices.data(), GL_STATIC_DRAW);

    GLuint uv_buffer;
    glGenBuffers(1, &uv_buffer);
    glBindBuffer(GL_ARRAY_BUFFER, uv_buffer);
    glBufferData(GL_ARRAY_BUFFER, uvs.size() * sizeof(glm::vec2), uvs.data(), GL_STATIC_DRAW);

    while (!glfwWindowShouldClose(window))
    {
        glClear(GL_COLOR_BUFFER_BIT | GL_DEPTH_BUFFER_BIT);

        glUseProgram(program);

        const auto [view, projection] = matrices_from_input(window);
        const auto mvp = projection * view * glm::mat4(1.0f);

        glUniformMatrix4fv(mvp_uniform, 1, GL_FALSE, glm::value_ptr(mvp));
        glActiveTexture(GL_TEXTURE0);
        glBindTexture(GL_TEXTURE_2D, texture_id);
        glUniform1i(texture_samp, 0);

        glEnableVertexAttribArray(0);
        glBindBuffer(GL_ARRAY_BUFFER, vertex_buffer);
        glVertexAttribPointer(0, 3, GL_FLOAT, GL_FALSE, 0, (void *)0);

        glEnableVertexAttribArray(1);
        glBindBuffer(GL_ARRAY_BUFFER, uv_buffer);
        glVertexAttribPointer(1, 2, GL_FLOAT, GL_FALSE, 0, (void *)0);

        glDrawArrays(GL_TRIANGLES, 0, vertices.size());

        glDisableVertexAttribArray(0);
        glDisableVertexAttribArray(1);

        glfwSwapBuffers(window);

        glfwPollEvents();

        if (glfwGetKey(window, GLFW_KEY_ESCAPE) == GLFW_PRESS)
            break;
    }

    glDeleteBuffers(1, &vertex_buffer);
    glDeleteBuffers(1, &uv_buffer);
    glDeleteTextures(1, &texture_id);
    glDeleteVertexArrays(1, &vertex_array);
    glDeleteProgram(program);

    glfwTerminate();
}