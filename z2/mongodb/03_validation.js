// mongo/03_validation.js
// Запуск: mongosh "mongodb://localhost:27017" mongo/03_validation.js
// Проверочные запросы: подсчёт документов и наличие "осиротевших" ссылок
// (аналог проверки внешних ключей, которую в PostgreSQL делает сам сервер).

db = db.getSiblingDB("faculty_db");

function countDocs(name) {
  print(name + ": " + db[name].countDocuments());
}

print("--- Количество документов ---");
["directions", "funding_types", "student_groups", "subjects", "teachers",
 "direction_subjects", "students", "enrollments", "grades", "lesson_slots",
 "lessons", "attendance"].forEach(countDocs);

print("\n--- Проверка ссылочной целостности (осиротевшие ссылки) ---");

function findOrphans(childCollection, foreignField, parentCollection) {
  const pipeline = [
    {
      $lookup: {
        from: parentCollection,
        localField: foreignField,
        foreignField: "_id",
        as: "parent"
      }
    },
    { $match: { parent: { $size: 0 } } },
    { $count: "orphans" }
  ];
  const result = db[childCollection].aggregate(pipeline).toArray();
  const orphans = result.length > 0 ? result[0].orphans : 0;
  print(
    childCollection + "." + foreignField + " -> " + parentCollection +
    ": осиротевших ссылок = " + orphans
  );
  return orphans;
}

let totalOrphans = 0;
totalOrphans += findOrphans("student_groups", "direction_id", "directions");
totalOrphans += findOrphans("direction_subjects", "direction_id", "directions");
totalOrphans += findOrphans("direction_subjects", "subject_id", "subjects");
totalOrphans += findOrphans("direction_subjects", "teacher_id", "teachers");
totalOrphans += findOrphans("enrollments", "student_id", "students");
totalOrphans += findOrphans("enrollments", "group_id", "student_groups");
totalOrphans += findOrphans("enrollments", "funding_type_id", "funding_types");
totalOrphans += findOrphans("grades", "enrollment_id", "enrollments");
totalOrphans += findOrphans("grades", "direction_subject_id", "direction_subjects");
totalOrphans += findOrphans("lessons", "group_id", "student_groups");
totalOrphans += findOrphans("lessons", "direction_subject_id", "direction_subjects");
totalOrphans += findOrphans("lessons", "slot_id", "lesson_slots");
totalOrphans += findOrphans("attendance", "lesson_id", "lessons");
totalOrphans += findOrphans("attendance", "enrollment_id", "enrollments");

print("\nИтого осиротевших ссылок: " + totalOrphans);
if (totalOrphans === 0) {
  print("Ссылочная целостность в норме.");
} else {
  print("ВНИМАНИЕ: найдены ссылки на несуществующие документы.");
}
