// ============================================================================
// FOUNDRY PLATFORM - MONGODB INITIALIZATION SCRIPT
// Creates indexes and seeds default superadmin user
// Usage:
//   mongosh "mongodb://127.0.0.1:27017/foundry" migrations/init.js
// ============================================================================

db.systems.createIndex({ "slug": 1 }, { unique: true });
db.system_configs.createIndex({ "system_id": 1, "key": 1 }, { unique: true });
db.models.createIndex({ "system_id": 1, "slug": 1 }, { unique: true });
db.model_fields.createIndex({ "model_id": 1, "name": 1 });
db.records.createIndex({ "system_id": 1, "model_id": 1, "id": 1 });
db.records.createIndex({ "data.$**": 1 });
db.admins.createIndex({ "username": 1 }, { unique: true });
db.admins.createIndex({ "email": 1 }, { unique: true });
db.audit_logs.createIndex({ "system_slug": 1 });
db.audit_logs.createIndex({ "created_at": -1 });
db.applications.createIndex({ "system_id": 1 });
db.applications.createIndex({ "token": 1 }, { unique: true });

// Seed default superadmin if not exists (admin / admin123456)
if (!db.admins.findOne({ username: "admin" })) {
    db.admins.insertOne({
        id: "00000000-0000-0000-0000-000000000001",
        username: "admin",
        email: "admin@foundry.local",
        password_hash: "$argon2id$v=19$m=19456,t=2,p=1$KX9WKigtvygJxZkV8V0k5w$Nf0fZMa6tRQWnFTqVO1xlFyFO/fzcvM1lmZ6hwnlD7Q",
        role: "super_admin",
        allowed_systems: ["*"],
        status: 1,
        created_at: new Date(),
        updated_at: new Date()
    });
    print("✅ Created default superadmin user (admin / admin123456)");
} else {
    print("ℹ️ Default superadmin already exists");
}
