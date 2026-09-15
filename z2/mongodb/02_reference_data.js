// mongo/02_reference_data.js
// Запуск: mongosh "mongodb://localhost:27017" mongo/02_reference_data.js
// Наполняет справочники: направления, группы, типы финансирования,
// предметы, преподавателей, учебный план и время пар.
// Скрипт идемпотентен: повторный запуск не создаёт дублей (используется
// findOneAndUpdate с upsert вместо insertOne).

db = db.getSiblingDB("faculty_db");

function upsertOne(collection, filter, doc) {
  return db[collection].findOneAndUpdate(
    filter,
    { $setOnInsert: doc },
    { upsert: true, returnDocument: "after" }
  );
}

// --- funding_types -----------------------------------------------------
const fundingTypes = [
  { code: "budget", name: "Бюджет", is_budget: true },
  { code: "contract", name: "Контракт", is_budget: false },
  { code: "target", name: "Целевое обучение", is_budget: true }
];
const fundingIds = {};
fundingTypes.forEach((item) => {
  const result = upsertOne("funding_types", { code: item.code }, item);
  fundingIds[item.code] = result._id;
});

// --- directions ----------------------------------------------------------
const directionNames = [
  "Программная инженерия",
  "Информационная безопасность",
  "Прикладная информатика",
  "Экономика",
  "Менеджмент"
];
const directionIds = {};
directionNames.forEach((name) => {
  const result = upsertOne("directions", { name: name }, { name: name });
  directionIds[name] = result._id;
});

// --- student_groups --------------------------------------------------------
const groups = [
  ["ПИ-101", "Программная инженерия"],
  ["ПИ-102", "Программная инженерия"],
  ["ПИ-201", "Программная инженерия"],
  ["ИБ-101", "Информационная безопасность"],
  ["ИБ-102", "Информационная безопасность"],
  ["ИБ-201", "Информационная безопасность"],
  ["ПР-101", "Прикладная информатика"],
  ["ПР-102", "Прикладная информатика"],
  ["ПР-201", "Прикладная информатика"],
  ["ЭК-101", "Экономика"],
  ["ЭК-102", "Экономика"],
  ["ЭК-201", "Экономика"],
  ["МН-101", "Менеджмент"],
  ["МН-102", "Менеджмент"],
  ["МН-201", "Менеджмент"]
];
const groupIds = {};
groups.forEach(([groupNumber, directionName]) => {
  const result = upsertOne(
    "student_groups",
    { group_number: groupNumber },
    { group_number: groupNumber, direction_id: directionIds[directionName] }
  );
  groupIds[groupNumber] = result._id;
});

// --- subjects --------------------------------------------------------------
const subjectNames = [
  "Базы данных",
  "Программирование",
  "Математика",
  "Операционные системы",
  "Компьютерные сети",
  "Информационная безопасность",
  "Экономическая теория",
  "Бухгалтерский учёт",
  "Менеджмент",
  "Маркетинг",
  "Статистика",
  "Проектирование информационных систем"
];
const subjectIds = {};
subjectNames.forEach((name) => {
  const result = upsertOne("subjects", { name: name }, { name: name });
  subjectIds[name] = result._id;
});

// --- teachers ----------------------------------------------------------
const teachers = [
  ["Иванов", "Иван", "Иванович"],
  ["Петрова", "Анна", "Сергеевна"],
  ["Сидоров", "Павел", "Алексеевич"],
  ["Смирнова", "Елена", "Олеговна"],
  ["Кузнецов", "Максим", "Викторович"],
  ["Волкова", "Мария", "Игоревна"],
  ["Морозов", "Дмитрий", "Андреевич"],
  ["Попова", "Ольга", "Романовна"],
  ["Орлов", "Никита", "Петрович"],
  ["Соколова", "Дарья", "Михайловна"]
];
const teacherIds = {};
teachers.forEach(([surname, firstName, patronymic]) => {
  const result = upsertOne(
    "teachers",
    { surname: surname, first_name: firstName, patronymic: patronymic },
    { surname: surname, first_name: firstName, patronymic: patronymic }
  );
  teacherIds[surname] = result._id;
});

// --- direction_subjects (учебный план) -------------------------------------
const plan = [
  ["Программная инженерия", "Базы данных", "Иванов"],
  ["Программная инженерия", "Программирование", "Петрова"],
  ["Программная инженерия", "Математика", "Сидоров"],
  ["Программная инженерия", "Операционные системы", "Смирнова"],
  ["Программная инженерия", "Проектирование информационных систем", "Кузнецов"],
  ["Информационная безопасность", "Базы данных", "Иванов"],
  ["Информационная безопасность", "Программирование", "Петрова"],
  ["Информационная безопасность", "Компьютерные сети", "Волкова"],
  ["Информационная безопасность", "Информационная безопасность", "Морозов"],
  ["Информационная безопасность", "Операционные системы", "Смирнова"],
  ["Прикладная информатика", "Базы данных", "Иванов"],
  ["Прикладная информатика", "Программирование", "Петрова"],
  ["Прикладная информатика", "Статистика", "Попова"],
  ["Прикладная информатика", "Менеджмент", "Орлов"],
  ["Прикладная информатика", "Проектирование информационных систем", "Кузнецов"],
  ["Экономика", "Математика", "Сидоров"],
  ["Экономика", "Экономическая теория", "Соколова"],
  ["Экономика", "Бухгалтерский учёт", "Попова"],
  ["Экономика", "Статистика", "Волкова"],
  ["Экономика", "Менеджмент", "Орлов"],
  ["Менеджмент", "Экономическая теория", "Соколова"],
  ["Менеджмент", "Бухгалтерский учёт", "Попова"],
  ["Менеджмент", "Менеджмент", "Орлов"],
  ["Менеджмент", "Маркетинг", "Кузнецов"],
  ["Менеджмент", "Статистика", "Волкова"]
];
plan.forEach(([directionName, subjectName, teacherSurname]) => {
  upsertOne(
    "direction_subjects",
    {
      direction_id: directionIds[directionName],
      subject_id: subjectIds[subjectName]
    },
    {
      direction_id: directionIds[directionName],
      subject_id: subjectIds[subjectName],
      teacher_id: teacherIds[teacherSurname]
    }
  );
});

// --- lesson_slots --------------------------------------------------------
const slots = [
  [1, "08:00", "09:30"],
  [2, "09:40", "11:10"],
  [3, "11:20", "12:50"],
  [4, "13:30", "15:00"],
  [5, "15:10", "16:40"],
  [6, "16:50", "18:20"]
];
slots.forEach(([pairNumber, startTime, endTime]) => {
  upsertOne(
    "lesson_slots",
    { pair_number: pairNumber },
    { pair_number: pairNumber, start_time: startTime, end_time: endTime }
  );
});

print("Справочные данные загружены: направления, группы, предметы, преподаватели, учебный план, время пар.");
