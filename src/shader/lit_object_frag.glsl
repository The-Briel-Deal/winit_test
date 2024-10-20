#version 460 core

struct Material {
    sampler2D diffuse;
    sampler2D specular;
    float shininess;
};

struct Light {
    vec3 position;

    vec3 ambient;
    vec3 diffuse;
    vec3 specular;

    float constant;
    float linear;
    float quadratic;
};

uniform Material material;
uniform Light light;

in vec3 FragPos;
in vec3 Normal;
in vec3 LightPos;
in vec2 TexCoords;

out vec4 FragColor;

vec3 calculateAmbientLighting(sampler2D diffuse, vec3 lightColor);
vec3 calculateDiffuseLighting(vec3 normal, vec3 lightDir, vec3 lightColor, sampler2D diffuse);
vec3 calculateSpecularLighting(vec3 normal, vec3 lightDir, vec3 lightColor, vec3 FragPos, sampler2D specularMap, float shininess);

void main()
{
    vec3 norm = normalize(Normal);
    vec3 lightDir = normalize(LightPos - FragPos);

    float distance = length(light.position - FragPos);
    float attenuation = 1.0 / (light.constant + light.linear * distance + light.quadratic * (pow(distance, 2)));

    vec3 ambientLighting = calculateAmbientLighting(material.diffuse, light.ambient) * attenuation;
    vec3 diffuseLighting = calculateDiffuseLighting(norm, lightDir, light.diffuse, material.diffuse) * attenuation;
    vec3 specularLighting = calculateSpecularLighting(norm, lightDir, light.specular, FragPos, material.specular, material.shininess) * attenuation;

    vec3 resultLighting = ambientLighting + diffuseLighting + specularLighting;

    FragColor = vec4(resultLighting, 1.0);
}

vec3 calculateAmbientLighting(sampler2D diffuse, vec3 lightColor) {
    vec3 ambientLighting = lightColor * vec3(texture(diffuse, TexCoords));
    return ambientLighting;
}

vec3 calculateDiffuseLighting(vec3 normal, vec3 lightDir, vec3 lightColor, sampler2D diffuse) {
    float diff = max(dot(normal, lightDir), 0.0);

    vec3 diffuseLighting = lightColor * diff * vec3(texture(diffuse, TexCoords));
    return diffuseLighting;
}

vec3 calculateSpecularLighting(vec3 normal, vec3 lightDir, vec3 lightColor, vec3 FragPos, sampler2D specularMap, float shininess) {
    vec3 viewDir = normalize(-FragPos);
    vec3 reflectDir = reflect(-lightDir, normal);
    float angleBetween = dot(viewDir, reflectDir);
    float spec = pow(max(angleBetween, 0.0), shininess);
    vec3 specularLighting = lightColor * spec * vec3(texture(specularMap, TexCoords));
    return specularLighting;
}
