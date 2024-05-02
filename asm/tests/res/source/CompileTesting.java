public class CompileTesting {
    // Test fields
    int field1 = 10;
    String field2 = "Hello, World!";
    boolean field3 = true;

    public static void main(String[] args) {
        // Test methods
        int result = addNumbers(5, 7);
        System.out.println("Result: " + result);

        // Test method invocation
        String message = "Hello";
        int length = message.length();

        // Test loop
        int[] numbers = { 1, 2, 3, 4, 5 };
        for (int number : numbers) {
            System.out.println(number);
        }
    }

    public static int addNumbers(int a, int b) {
        return a + b;
    }
}
