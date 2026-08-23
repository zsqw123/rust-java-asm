package mapping.fixture;

public final class MappingEntry {
    private final UserService service = new UserService(new UserRepository());

    public static void main(String[] args) {
        MappingEntry entry = new MappingEntry();
        System.out.println(entry.describe(7));
        System.out.println(entry.overloaded(2));
        System.out.println(entry.overloaded("mapping"));
    }

    public String describe(int userId) {
        User user = service.findUser(userId);
        String label = user.displayName();
        return user.id() + ":" + label;
    }

    public String overloaded(int repeat) {
        return "number-" + repeat;
    }

    public String overloaded(String value) {
        return "text-" + value;
    }

    public User[] copyUsers(User[] users) {
        return users.clone();
    }
}
