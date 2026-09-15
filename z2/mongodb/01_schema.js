// mongo/01_schema.js
// Запуск:
// mongosh "mongodb://localhost:27017" mongo/01_schema.js
//
// Создаёт БД faculty_db, коллекции и валидаторы $jsonSchema.
//
// $jsonSchema контролирует типы, обязательные поля и допустимые значения.
// MongoDB не поддерживает FOREIGN KEY как constraint: целостность ObjectId-
// ссылок проверяется прикладным кодом и скриптом 03_validation.js.

db = db.getSiblingDB("faculty_db");

db.createCollection("directions", {
  validator: {
    $jsonSchema: {
      bsonType: "object",
      required: ["name"],
      additionalProperties: false,
      properties: {
        _id: { bsonType: "objectId" },
        name: { bsonType: "string", minLength: 1, maxLength: 200 }
      }
    }
  },
  validationLevel: "strict",
  validationAction: "error"
});
db.directions.createIndex({ name: 1 }, { unique: true });

db.createCollection("funding_types", {
  validator: {
    $jsonSchema: {
      bsonType: "object",
      required: ["code", "name", "is_budget"],
      additionalProperties: false,
      properties: {
        _id: { bsonType: "objectId" },
        code: { bsonType: "string", minLength: 1, maxLength: 30 },
        name: { bsonType: "string", minLength: 1, maxLength: 100 },
        is_budget: { bsonType: "bool" }
      }
    }
  },
  validationLevel: "strict",
  validationAction: "error"
});
db.funding_types.createIndex({ code: 1 }, { unique: true });
db.funding_types.createIndex({ name: 1 }, { unique: true });

db.createCollection("student_groups", {
  validator: {
    $jsonSchema: {
      bsonType: "object",
      required: ["group_number", "direction_id"],
      additionalProperties: false,
      properties: {
        _id: { bsonType: "objectId" },
        group_number: { bsonType: "string", minLength: 1, maxLength: 30 },

        // ObjectId из directions — аналог FOREIGN KEY.
        // MongoDB не проверяет существование документа автоматически.
        direction_id: { bsonType: "objectId" }
      }
    }
  },
  validationLevel: "strict",
  validationAction: "error"
});
db.student_groups.createIndex({ group_number: 1 }, { unique: true });
db.student_groups.createIndex({ direction_id: 1 });

db.createCollection("teachers", {
  validator: {
    $jsonSchema: {
      bsonType: "object",
      required: ["surname", "first_name"],
      additionalProperties: false,
      properties: {
        _id: { bsonType: "objectId" },
        surname: { bsonType: "string", minLength: 1, maxLength: 100 },
        first_name: { bsonType: "string", minLength: 1, maxLength: 100 },
        patronymic: { bsonType: ["string", "null"], maxLength: 100 }
      }
    }
  },
  validationLevel: "strict",
  validationAction: "error"
});
db.teachers.createIndex(
  { surname: 1, first_name: 1, patronymic: 1 },
  { unique: true }
);

db.createCollection("subjects", {
  validator: {
    $jsonSchema: {
      bsonType: "object",
      required: ["name"],
      additionalProperties: false,
      properties: {
        _id: { bsonType: "objectId" },
        name: { bsonType: "string", minLength: 1, maxLength: 200 }
      }
    }
  },
  validationLevel: "strict",
  validationAction: "error"
});
db.subjects.createIndex({ name: 1 }, { unique: true });

db.createCollection("direction_subjects", {
  validator: {
    $jsonSchema: {
      bsonType: "object",
      required: ["direction_id", "subject_id", "teacher_id"],
      additionalProperties: false,
      properties: {
        _id: { bsonType: "objectId" },
        direction_id: { bsonType: "objectId" },
        subject_id: { bsonType: "objectId" },
        teacher_id: { bsonType: "objectId" }
      }
    }
  },
  validationLevel: "strict",
  validationAction: "error"
});
db.direction_subjects.createIndex(
  { direction_id: 1, subject_id: 1 },
  { unique: true }
);
db.direction_subjects.createIndex({ teacher_id: 1 });
db.direction_subjects.createIndex({ subject_id: 1 });

db.createCollection("students", {
  validator: {
    $jsonSchema: {
      bsonType: "object",
      required: [
        "surname",
        "first_name",
        "birth_date",
        "email",
        "addresses",
        "phones"
      ],
      additionalProperties: false,
      properties: {
        _id: { bsonType: "objectId" },

        surname: {
          bsonType: "string",
          minLength: 1,
          maxLength: 100
        },

        first_name: {
          bsonType: "string",
          minLength: 1,
          maxLength: 100
        },

        patronymic: {
          bsonType: ["string", "null"],
          maxLength: 100
        },

        birth_date: {
          bsonType: "date"
        },

        email: {
          bsonType: "string",
          minLength: 3,
          maxLength: 254
        },

        // Один студент может иметь несколько адресов.
        // Адреса являются embedded-документами: они принадлежат только
        // студенту и обычно загружаются вместе с его карточкой.
        addresses: {
          bsonType: "array",
          minItems: 1,
          items: {
            bsonType: "object",
            required: [
              "kind",
              "city",
              "street",
              "house",
              "is_primary"
            ],
            additionalProperties: false,
            properties: {
              kind: {
                bsonType: "string",
                enum: [
                  "registration",
                  "temporary",
                  "home",
                  "work"
                ]
              },

              city: {
                bsonType: "string",
                minLength: 1,
                maxLength: 100
              },

              street: {
                bsonType: "string",
                minLength: 1,
                maxLength: 150
              },

              house: {
                bsonType: "string",
                minLength: 1,
                maxLength: 20
              },

              is_primary: {
                bsonType: "bool"
              }
            }
          }
        },

        phones: {
          bsonType: "array",
          minItems: 1,
          items: {
            bsonType: "string",
            minLength: 1,
            maxLength: 30
          }
        }
      }
    }
  },
  validationLevel: "strict",
  validationAction: "error"
});
db.students.createIndex({ email: 1 }, { unique: true });
db.students.createIndex({
  surname: 1,
  first_name: 1,
  patronymic: 1
});
db.students.createIndex({ birth_date: 1 });

db.createCollection("enrollments", {
  validator: {
    $jsonSchema: {
      bsonType: "object",
      required: [
        "student_id",
        "group_id",
        "funding_type_id"
      ],
      additionalProperties: false,
      properties: {
        _id: { bsonType: "objectId" },
        student_id: { bsonType: "objectId" },
        group_id: { bsonType: "objectId" },
        funding_type_id: { bsonType: "objectId" }
      }
    }
  },
  validationLevel: "strict",
  validationAction: "error"
});
db.enrollments.createIndex(
  { student_id: 1, group_id: 1 },
  { unique: true }
);
db.enrollments.createIndex({ group_id: 1 });
db.enrollments.createIndex({ funding_type_id: 1 });

db.createCollection("grades", {
  validator: {
    $jsonSchema: {
      bsonType: "object",
      required: [
        "enrollment_id",
        "direction_subject_id"
      ],
      additionalProperties: false,
      properties: {
        _id: { bsonType: "objectId" },
        enrollment_id: { bsonType: "objectId" },
        direction_subject_id: { bsonType: "objectId" },
        grade: {
          bsonType: ["int", "null"],
          minimum: 2,
          maximum: 5
        },
        exam_date: {
          bsonType: ["date", "null"]
        }
      }
    }
  },
  validationLevel: "strict",
  validationAction: "error"
});
db.grades.createIndex(
  { enrollment_id: 1, direction_subject_id: 1 },
  { unique: true }
);

db.createCollection("lesson_slots", {
  validator: {
    $jsonSchema: {
      bsonType: "object",
      required: [
        "pair_number",
        "start_time",
        "end_time"
      ],
      additionalProperties: false,
      properties: {
        _id: { bsonType: "objectId" },
        pair_number: {
          bsonType: "int",
          minimum: 1
        },
        start_time: {
          bsonType: "string",
          pattern: "^[0-2][0-9]:[0-5][0-9]$"
        },
        end_time: {
          bsonType: "string",
          pattern: "^[0-2][0-9]:[0-5][0-9]$"
        }
      }
    }
  },
  validationLevel: "strict",
  validationAction: "error"
});
db.lesson_slots.createIndex(
  { pair_number: 1 },
  { unique: true }
);

db.createCollection("lessons", {
  validator: {
    $jsonSchema: {
      bsonType: "object",
      required: [
        "group_id",
        "direction_subject_id",
        "lesson_date",
        "slot_id"
      ],
      additionalProperties: false,
      properties: {
        _id: { bsonType: "objectId" },
        group_id: { bsonType: "objectId" },
        direction_subject_id: { bsonType: "objectId" },
        lesson_date: { bsonType: "date" },
        slot_id: { bsonType: "objectId" }
      }
    }
  },
  validationLevel: "strict",
  validationAction: "error"
});
db.lessons.createIndex(
  { group_id: 1, lesson_date: 1, slot_id: 1 },
  { unique: true }
);
db.lessons.createIndex({ direction_subject_id: 1 });

db.createCollection("attendance", {
  validator: {
    $jsonSchema: {
      bsonType: "object",
      required: [
        "lesson_id",
        "enrollment_id",
        "attended"
      ],
      additionalProperties: false,
      properties: {
        _id: { bsonType: "objectId" },
        lesson_id: { bsonType: "objectId" },
        enrollment_id: { bsonType: "objectId" },
        attended: { bsonType: "bool" }
      }
    }
  },
  validationLevel: "strict",
  validationAction: "error"
});
db.attendance.createIndex(
  { lesson_id: 1, enrollment_id: 1 },
  { unique: true }
);
db.attendance.createIndex({ enrollment_id: 1 });

print("Схема faculty_db создана: коллекции и индексы готовы.");