#define _POSIX_C_SOURCE 200809L
#include "probe.h"
#include <vulkan/vulkan.h>

#define LABEL "I01_MESA_VULKAN_STARTUP"
#define CHECK(condition, stage) do { if (!(condition)) { \
    fprintf(stderr, LABEL "_FAIL stage=%s vk=%d\n", stage, status); goto done; } } while (0)
#define CALL(expression, stage) do { status = (expression); \
    CHECK(status == VK_SUCCESS, stage); } while (0)

int main(int argc, char **argv)
{
    setvbuf(stdout, NULL, _IONBF, 0);
    if (argc > 2) { fprintf(stderr, "usage: %s [/dev/dri/renderD128]\n", argv[0]); return 2; }
    if (probe_environment(LABEL)) return 1;
    int fd = probe_drm_open(LABEL, argc == 2 ? argv[1] : "/dev/dri/renderD128");
    if (fd < 0) return 1;
    int result = 1;
    VkResult status = VK_SUCCESS;
    VkInstance instance = VK_NULL_HANDLE;
    VkDevice device = VK_NULL_HANDLE;
    VkPhysicalDevice *physical = NULL;
    VkQueueFamilyProperties *families = NULL;
    uint32_t loader_version = 0, count = 0, family_count = 0;
    CALL(vkEnumerateInstanceVersion(&loader_version), "vkEnumerateInstanceVersion");
    printf("VULKAN_LOADER=%u.%u.%u\n", VK_API_VERSION_MAJOR(loader_version),
           VK_API_VERSION_MINOR(loader_version), VK_API_VERSION_PATCH(loader_version));
    CHECK(loader_version >= VK_API_VERSION_1_2, "Vulkan_1_2_loader");
    VkApplicationInfo app = {.sType = VK_STRUCTURE_TYPE_APPLICATION_INFO,
        .pApplicationName = "WebBoxVM stock Mesa startup", .apiVersion = VK_API_VERSION_1_2};
    VkInstanceCreateInfo create = {.sType = VK_STRUCTURE_TYPE_INSTANCE_CREATE_INFO,
                                  .pApplicationInfo = &app};
    CALL(vkCreateInstance(&create, NULL, &instance), "vkCreateInstance");
    CALL(vkEnumeratePhysicalDevices(instance, &count, NULL), "vkEnumeratePhysicalDevices_count");
    CHECK(count > 0 && count <= 16, "physical_device_count");
    physical = calloc(count, sizeof(*physical));
    CHECK(physical, "physical_device_allocation");
    CALL(vkEnumeratePhysicalDevices(instance, &count, physical), "vkEnumeratePhysicalDevices");
    CHECK(count == 1, "single_pinned_venus_device");
    VkPhysicalDeviceProperties basic;
    vkGetPhysicalDeviceProperties(physical[0], &basic);
    CHECK(basic.apiVersion >= VK_API_VERSION_1_2, "Vulkan_1_2_driver_properties");
    VkPhysicalDeviceDriverProperties driver = {
        .sType = VK_STRUCTURE_TYPE_PHYSICAL_DEVICE_DRIVER_PROPERTIES};
    VkPhysicalDeviceProperties2 properties = {
        .sType = VK_STRUCTURE_TYPE_PHYSICAL_DEVICE_PROPERTIES_2, .pNext = &driver};
    vkGetPhysicalDeviceProperties2(physical[0], &properties);
    probe_text("VULKAN_DEVICE", properties.properties.deviceName);
    probe_text("VULKAN_DRIVER", driver.driverName);
    probe_text("VULKAN_DRIVER_INFO", driver.driverInfo);
    uint32_t api = properties.properties.apiVersion;
    printf("VULKAN_API=%u.%u.%u DRIVER_ID=%u DEVICE_TYPE=%u\n", VK_API_VERSION_MAJOR(api),
        VK_API_VERSION_MINOR(api), VK_API_VERSION_PATCH(api), driver.driverID,
        properties.properties.deviceType);
    CHECK(driver.driverID == VK_DRIVER_ID_MESA_VENUS && !strcmp(driver.driverName, "venus") &&
          probe_mesa_version(driver.driverInfo), "pinned_venus_identity");
    CHECK(api >= VK_API_VERSION_1_2 && properties.properties.deviceType != VK_PHYSICAL_DEVICE_TYPE_CPU &&
          !probe_software(properties.properties.deviceName), "hardware_Vulkan_device");
    vkGetPhysicalDeviceQueueFamilyProperties(physical[0], &family_count, NULL);
    CHECK(family_count > 0 && family_count <= 256, "queue_family_count");
    families = calloc(family_count, sizeof(*families));
    CHECK(families, "queue_family_allocation");
    vkGetPhysicalDeviceQueueFamilyProperties(physical[0], &family_count, families);
    uint32_t family = 0;
    while (family < family_count &&
           !(families[family].queueCount && (families[family].queueFlags & VK_QUEUE_GRAPHICS_BIT))) ++family;
    CHECK(family < family_count, "graphics_queue_family");
    float priority = 1.0f;
    VkDeviceQueueCreateInfo queue = {.sType = VK_STRUCTURE_TYPE_DEVICE_QUEUE_CREATE_INFO,
        .queueFamilyIndex = family, .queueCount = 1, .pQueuePriorities = &priority};
    VkDeviceCreateInfo device_create = {.sType = VK_STRUCTURE_TYPE_DEVICE_CREATE_INFO,
        .queueCreateInfoCount = 1, .pQueueCreateInfos = &queue};
    CALL(vkCreateDevice(physical[0], &device_create, NULL, &device), "vkCreateDevice");
    VkQueue actual_queue = VK_NULL_HANDLE;
    vkGetDeviceQueue(device, family, 0, &actual_queue);
    CHECK(actual_queue != VK_NULL_HANDLE, "vkGetDeviceQueue");
    CALL(vkDeviceWaitIdle(device), "vkDeviceWaitIdle");
    printf("VULKAN_GRAPHICS_QUEUE_FAMILY=%u\n", family);
    result = 0;
done:
    if (device != VK_NULL_HANDLE) vkDestroyDevice(device, NULL);
    if (instance != VK_NULL_HANDLE) vkDestroyInstance(instance, NULL);
    free(families);
    free(physical);
    if (close(fd)) { fprintf(stderr, LABEL "_FAIL stage=close errno=%d\n", errno); result = 1; }
    if (!result) puts(LABEL "_PASS");
    return result;
}
