// ============================================================================
// Foundry Migration: v0.2.0_add_applications (MongoDB)
// Upstream Framework: Foundry v0.2.0
// Description: Create applications collection, indexes, and register in _foundry_migrations
// Safe for manual DBA execution with self-recording metadata tracking
// ============================================================================
db = db.getSiblingDB("foundry");
if (!db.getCollectionNames().includes("applications")) {
    db.createCollection("applications");
}
db.applications.createIndex({ "system_id": 1 });
db.applications.createIndex({ "app_key": 1 }, { unique: true });
db.applications.createIndex({ "app_secret": 1 });

// Migration Metadata Tracking
if (!db.getCollectionNames().includes("_foundry_migrations")) {
    db.createCollection("_foundry_migrations");
}
db._foundry_migrations.updateOne(
    { sequence: 2 },
    { $set: { sequence: 2, version: "v0.2.0", name: "add_applications", applied_at: new Date() } },
    { upsert: true }
);
