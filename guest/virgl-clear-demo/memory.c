void *memset(void *destination, int value, unsigned long count)
{
    unsigned char *bytes = destination;

    while (count != 0) {
        *bytes++ = (unsigned char)value;
        count--;
    }
    return destination;
}

void *memcpy(void *destination, const void *source, unsigned long count)
{
    unsigned char *out = destination;
    const unsigned char *in = source;

    while (count != 0) {
        *out++ = *in++;
        count--;
    }
    return destination;
}
