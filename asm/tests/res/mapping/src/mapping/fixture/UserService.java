package mapping.fixture;

public final class UserService {
    private final UserRepository repository;

    public UserService(UserRepository repository) {
        this.repository = repository;
    }

    public User findUser(int userId) {
        if (userId <= 0) {
            throw new IllegalArgumentException("userId must be positive");
        }
        User loaded = repository.load(userId);
        return normalize(loaded);
    }

    private User normalize(User user) {
        String value = user.displayName().trim();
        String normalized = value.toUpperCase();
        return new User(user.id(), normalized);
    }
}
