#version 330 core

layout (points) in;
layout (triangle_strip, max_vertices = 5) out;

uniform float rowCount;
uniform float columnCount;

float init(float index, float count){
    return index * (2.0 / count) - 1.0;
}

void main(){
    vec4 indices = gl_in[0].gl_Position;

    float x_one = init(indices.x, columnCount);
    float y_one = - 1 * init(indices.y, rowCount);
    float x_two = init(indices.x + 1.0, columnCount);
    float y_two = -1 * init(indices.y + 1.0, rowCount);

    gl_Position = vec4(x_one, y_one, 0.0, 1.0);
    EmitVertex();

    gl_Position = vec4(x_two, y_one, 0.0, 1.0);
    EmitVertex();

    gl_Position = vec4(x_two, y_two, 0.0, 1.0);
    EmitVertex();

    gl_Position = vec4(x_one, y_two, 0.0, 1.0);
    EmitVertex();

    gl_Position = vec4(x_one, y_one, 0.0, 1.0);
    EmitVertex();

    EndPrimitive();
}

