package mapping.fixture;

public final class UserRepository {
    public User load(int userId) {
        return new User(userId, "user-" + userId);
    }
}
