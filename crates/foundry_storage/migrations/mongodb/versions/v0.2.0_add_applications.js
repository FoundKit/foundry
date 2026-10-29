// ============================================================================
// Foundry Migration: v0.2.0_add_applications (MongoDB)
// Upstream Framework: Foundry v0.2.0
// Description: Create applications collection and indexes
// ============================================================================
db = db.getSiblingDB("foundry");
if (!db.getCollectionNames().includes("applications")) {
    db.createCollection("applications");
}
db.applications.createIndex({ "system_id": 1 });
db.applications.createIndex({ "app_key": 1 }, { unique: true });
db.applications.createIndex({ "app_secret": 1 });
