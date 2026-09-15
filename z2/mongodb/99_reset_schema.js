// mongo/99_reset_schema.js
// Запуск: mongosh "mongodb://localhost:27017" mongo/99_reset_schema.js
// Полностью удаляет базу faculty_db. Используется только для осознанной
// пересборки — все коллекции, документы, валидаторы и индексы теряются.

db = db.getSiblingDB("faculty_db");
db.dropDatabase();
print("База faculty_db удалена.");
