#include <stdio.h>

// Macro definition (Preprocessor)
#define MAX_ITEMS 5

/*
 * Struct definition to test composite types,
 * pointers, and field access.
 */
struct Vector2D {
    float x;
    float y;
};

// Function declaration
int calculate_sum(int numbers[], int count);

int main(void) {
    // Basic types and variable declarations
    int integers[MAX_ITEMS] = {10, 20, 30, 40, 50};
    char greeting[] = "Hello, Compiler!";
    struct Vector2D position = { 3.14f, 2.71f };
    struct Vector2D *pos_ptr = &position;

    // Control flow: Loop & Conditionals
    int total = 0;
    for (int i = 0; i < MAX_ITEMS; i++) {
        if (integers[i] % 20 == 0) {
            total += integers[i] * 2;
        } else {
            total += integers[i];
        }
    }

    // Pointer arrow operator access
    pos_ptr->x += 1.0f;

    // String formatting & output
    printf("%s\n", greeting);
    printf("Calculated Total: %d\n", total);
    printf("Position: (%.2f, %.2f)\n", pos_ptr->x, pos_ptr->y);

    return 0;
}

// Function implementation
int calculate_sum(int numbers[], int count) {
    int sum = 0;
    int i = 0;
    while (i < count) {
        sum += numbers[i];
        i++;
    }
    return sum;
}
