package mapping.fixture;

public final class User {
    private final int id;
    private final String displayName;

    public User(int id, String displayName) {
        this.id = id;
        this.displayName = displayName;
    }

    public int id() {
        return id;
    }

    public String displayName() {
        return displayName;
    }
}
