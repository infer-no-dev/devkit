/**
 * Sorts an array in ascending order.
 *
 * @param {Array} arr The input array to be sorted.
 * @returns {Array} A new sorted array.
 */
async function sortArray(arr) {
    return arr.sort((a, b) => a - b);
}

/**
 * Creates and returns a sample project with a test environment for Agentic Dev Env.
 *
 * @param {string} [projectName] The name of the project. Defaults to "sample-project".
 * @returns {TestEnvironment} A new TestEnvironment instance.
 */
async function createSampleProject(projectName = 'sample-project') {
    // Implementation not shown in provided codebase context
}

/**
 * Creates and runs a sample test environment for Agentic Dev Env.
 *
 * @param {string} [projectName] The name of the project. Defaults to "sample-project".
 * @returns {void}
 */
async function main(projectName = 'sample-project') {
    const testEnvironment = await createSampleProject(projectName);
    console.log(`Test environment created: ${testEnvironment.name}`);
}

/**
 * Adds a command to add a new sample project.
 *
 * @param {string} [projectName] The name of the project. Defaults to "sample-project".
 * @returns {void}
 */
async function add(projectName = 'sample-project') {
    console.log(`Adding sample project: ${projectName}`);
}

// Usage
main('new-sample-project')
  .then(() => add('new-sample-project'))
  .catch((err) => console.error(err));

// --- Generated Code ---
/**
 * Simple Bubble Sort Function
 *
 * @param {Array} arr - The input array to be sorted.
 * @returns {Array} The sorted array.
 */
async function bubbleSort(arr) {
    let len = arr.length;
    for (let i = 0; i < len - 1; i++) {
        for (let j = 0; j < len - i - 1; j++) {
            if (arr[j] > arr[j + 1]) {
                // Swap elements
                [arr[j], arr[j + 1]] = [arr[j + 1], arr[j]];
            }
        }
    }

    return arr;
}

/**
 * Test Bubble Sort Function
 */
async function testBubbleSort() {
    const arr = [5, 2, 8, 3, 1, 4];
    const sortedArr = await bubbleSort(arr);

    console.log("Sorted Array:", sortedArr);
}

// Run the test
await testBubbleSort();