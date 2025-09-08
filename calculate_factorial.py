import math

def calculate_factorial(n: int) -> int:
    """
    Calculates the factorial of a given number.

    Args:
        n (int): The number to calculate the factorial for.

    Returns:
        int: The calculated factorial.
    """
    if not isinstance(n, int):
        raise ValueError("Input must be an integer.")

    # Use math.factorial function
    return math.factorial(n)

# Alternatively, you can implement it manually
def calculate_factorial_manual(n: int) -> int:
    """
    Calculates the factorial of a given number manually.

    Args:
        n (int): The number to calculate the factorial for.

    Returns:
        int: The calculated factorial.
    """
    if not isinstance(n, int):
        raise ValueError("Input must be an integer.")

    result = 1
    for i in range(2, n + 1):
        result *= i

    return result