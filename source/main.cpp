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

GLuint loadBMP(std::string_view imagepath)
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
    glfwWindowHint(GLFW_CONTEXT_VERSION_MINOR, 4);
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

    constexpr auto vertex_shader_text = R"(
        #version 330 core
        layout(location = 0) in vec3 vertex_pos;
        layout(location = 1) in vec2 vertex_uv;

        out vec2 uv;

        uniform mat4 MVP;

        void main()
        {  
            gl_Position = MVP * vec4(vertex_pos, 1.0);
            uv = vertex_uv;
        }
        )";
    constexpr auto fragment_shader_text = R"(
        #version 330 core
        
        in vec2 uv;
        
        out vec3 color;
        
        uniform sampler2D texture_samp;

        void main(){
          color = texture( texture_samp, uv ).rgb;
        }
        )";

    const auto vertex_shader = glCreateShader(GL_VERTEX_SHADER);
    glShaderSource(vertex_shader, 1, &vertex_shader_text, nullptr);
    glCompileShader(vertex_shader);
    const auto fragment_shader = glCreateShader(GL_FRAGMENT_SHADER);
    glShaderSource(fragment_shader, 1, &fragment_shader_text, nullptr);
    glCompileShader(fragment_shader);
    const auto program = glCreateProgram();
    glAttachShader(program, vertex_shader);
    glAttachShader(program, fragment_shader);
    glLinkProgram(program);

    glDetachShader(program, vertex_shader);
    glDetachShader(program, fragment_shader);
    glDeleteShader(vertex_shader);
    glDeleteShader(fragment_shader);

    constexpr float vertices[] = {
        -1.0f, -1.0f, 0.0f,
        1.0f, -1.0f, 0.0f,
        0.0f, 1.0f, 0.0f,

        //cube
        /*
        -1.0f, -1.0f, -1.0f,
        -1.0f, -1.0f, 1.0f,
        -1.0f, 1.0f, 1.0f,
        1.0f, 1.0f, -1.0f,
        -1.0f, -1.0f, -1.0f,
        -1.0f, 1.0f, -1.0f,
        1.0f, -1.0f, 1.0f,
        -1.0f, -1.0f, -1.0f,
        1.0f, -1.0f, -1.0f,
        1.0f, 1.0f, -1.0f,
        1.0f, -1.0f, -1.0f,
        -1.0f, -1.0f, -1.0f,
        -1.0f, -1.0f, -1.0f,
        -1.0f, 1.0f, 1.0f,
        -1.0f, 1.0f, -1.0f,
        1.0f, -1.0f, 1.0f,
        -1.0f, -1.0f, 1.0f,
        -1.0f, -1.0f, -1.0f,
        -1.0f, 1.0f, 1.0f,
        -1.0f, -1.0f, 1.0f,
        1.0f, -1.0f, 1.0f,
        1.0f, 1.0f, 1.0f,
        1.0f, -1.0f, -1.0f,
        1.0f, 1.0f, -1.0f,
        1.0f, -1.0f, -1.0f,
        1.0f, 1.0f, 1.0f,
        1.0f, -1.0f, 1.0f,
        1.0f, 1.0f, 1.0f,
        1.0f, 1.0f, -1.0f,
        -1.0f, 1.0f, -1.0f,
        1.0f, 1.0f, 1.0f,
        -1.0f, 1.0f, -1.0f,
        -1.0f, 1.0f, 1.0f,
        1.0f, 1.0f, 1.0f,
        -1.0f, 1.0f, 1.0f,
        1.0f, -1.0f, 1.0f
        */
    };

    constexpr float uv_coords[] = {
        0.0f,
        0.0f,
        1.0f,
        0.0f,
        0.5f,
        1.0f,
    };
    GLuint vertex_array;
    glGenVertexArrays(1, &vertex_array);
    glBindVertexArray(vertex_array);

    GLuint vertex_buffer;
    glGenBuffers(1, &vertex_buffer);
    glBindBuffer(GL_ARRAY_BUFFER, vertex_buffer);
    glBufferData(GL_ARRAY_BUFFER, sizeof(vertices), vertices, GL_STATIC_DRAW);

    GLuint color_buffer;
    glGenBuffers(1, &color_buffer);
    glBindBuffer(GL_ARRAY_BUFFER, color_buffer);
    glBufferData(GL_ARRAY_BUFFER, sizeof(uv_coords), uv_coords, GL_STATIC_DRAW);

    const auto texture = loadBMP("textures/uvtemplate.bmp");

    const auto mvp_id = glGetUniformLocation(program, "MVP");

    while (!glfwWindowShouldClose(window))
    {
        glClear(GL_COLOR_BUFFER_BIT | GL_DEPTH_BUFFER_BIT);

        glUseProgram(program);

        const auto [view, projection] = matrices_from_input(window);
        const auto mvp = projection * view * glm::mat4(1.0f);

        glUniformMatrix4fv(mvp_id, 1, GL_FALSE, glm::value_ptr(mvp));

        glEnableVertexAttribArray(0);
        glBindBuffer(GL_ARRAY_BUFFER, vertex_buffer);
        glVertexAttribPointer(0, 3, GL_FLOAT, GL_FALSE, 0, (void *)0);

        glEnableVertexAttribArray(1);
        glBindBuffer(GL_ARRAY_BUFFER, color_buffer);
        glVertexAttribPointer(1, 2, GL_FLOAT, GL_FALSE, 0, (void *)0);

        glDrawArrays(GL_TRIANGLES, 0, sizeof(vertices) / sizeof(float) / 3);

        glDisableVertexAttribArray(0);
        glDisableVertexAttribArray(1);

        glfwSwapBuffers(window);

        glfwPollEvents();

        if (glfwGetKey(window, GLFW_KEY_ESCAPE) == GLFW_PRESS)
            break;
    }

    glDeleteBuffers(1, &vertex_buffer);
    glDeleteBuffers(1, &color_buffer);
    glDeleteTextures(1, &texture);
    glDeleteVertexArrays(1, &vertex_array);
    glDeleteProgram(program);

    glfwTerminate();
}