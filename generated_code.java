import java.util.regex.Matcher;
import java.util.regex.Pattern;

/**
 * Utility class for validating email addresses.
 */
public class EmailValidator {

    private static final String EMAIL_REGEX = "^[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\\.[a-zA-Z]{2,}$";

    /**
     * Validate an email address using a regular expression.
     *
     * @param email the email address to validate
     * @return true if the email is valid, false otherwise
     */
    public static boolean isValidEmail(String email) {
        Pattern pattern = Pattern.compile(EMAIL_REGEX);
        Matcher matcher = pattern.matcher(email);
        return matcher.matches();
    }

    /**
     * Main function for testing purposes.
     *
     * @param args command line arguments (not used)
     */
    public static void main(String[] args) {
        String validEmail = "test@example.com";
        String invalidEmail = "invalid";

        System.out.println("Is '" + validEmail + "' a valid email? " + isValidEmail(validEmail));
        System.out.println("Is '" + invalidEmail + "' a valid email? " + isValidEmail(invalidEmail));
    }
}