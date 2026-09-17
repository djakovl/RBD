use anyhow::Result;
use postgres::{types::ToSql, Client};

use crate::{queries, selectors, table};

pub const MENU: &[(&str, &str)] = &[
    ("1", "[Ф1] Средняя оценка по предмету для группы"),
    ("2", "[Ф2] Средняя оценка по предмету для направления"),
    ("3", "[Ф3] Количество отличников в группе"),
    ("4", "[Ф4] Есть ли у студента несданные экзамены"),
    ("5", "[Ф5] Пропуски по каждому студенту"),
    ("6", "[Ф6] Пропуски по каждой группе"),
    ("7", "[Ф7] Пропуски по каждому направлению"),
    ("8", "[Ф8] Пропуски по каждому преподавателю"),
    ("9", "[Тр] Таблица: среднее группы по предмету"),
    ("10", "[Тр] Таблица: среднее группы по всем предметам"),
    ("11", "[Тр] Таблица: среднее по предмету глобально"),
    ("12", "[Тр] Тест: недопустимая оценка (ожидаем ошибку)"),
    ("13", "[Тр] Тест: оценка НЕ своим преподавателем (ожидаем ошибку)"),
    ("14", "[Тр] Тест: оценка своим преподавателем (проходит)"),
    ("15", "[Пар] Числовой доп. параметр: среднее и сумма по группе"),
    ("16", "[Пар] Текстовый доп. параметр: поиск значения"),
    ("99", "[ЛОМАЕМ БД] Демонстрация самозацикливания триггера"),
];

fn show(client: &mut Client, sql: &str, params: &[&(dyn ToSql + Sync)]) -> Result<()> {
    let rows = client.query(sql, params)?;
    table::print_rows(&rows);
    Ok(())
}

pub fn run(client: &mut Client, choice: &str) -> Result<()> {
    match choice {
        // ---------------- Задание 3: функции ----------------
        "1" => {
            let group_id = selectors::group(client)?;
            let subject_id = selectors::subject(client)?;
            show(client, queries::F01_AVG_GRADE_GROUP_SUBJECT, &[&group_id, &subject_id])
        }
        "2" => {
            let direction_id = selectors::direction(client)?;
            let subject_id = selectors::subject(client)?;
            show(client, queries::F02_AVG_GRADE_DIRECTION_SUBJECT, &[&direction_id, &subject_id])
        }
        "3" => {
            let group_id = selectors::group(client)?;
            show(client, queries::F03_COUNT_EXCELLENT_STUDENTS, &[&group_id])
        }
        "4" => {
            let student_id = selectors::any_student(client)?;
            show(client, queries::F04_HAS_FAILED_EXAMS, &[&student_id])
        }
        "5" => show(client, queries::F05_MISSED_BY_STUDENT, &[]),
        "6" => show(client, queries::F06_MISSED_BY_GROUP, &[]),
        "7" => show(client, queries::F07_MISSED_BY_DIRECTION, &[]),
        "8" => show(client, queries::F08_MISSED_BY_TEACHER, &[]),

        // ---------------- Задание 3: триггеры (просмотр/тесты) ----------------
        "9" => show(client, queries::T09_GROUP_SUBJECT_AVG, &[]),
        "10" => show(client, queries::T10_GROUP_OVERALL_AVG, &[]),
        "11" => show(client, queries::T11_SUBJECT_AVG, &[]),
        "12" => {
            let enrollment_id = selectors::read_i32("ID зачисления (enrollments.id), например 1: ", 1, 1_000_000)? as i64;
            match client.execute(queries::T14_INSERT_INVALID_GRADE, &[&enrollment_id]) {
                Ok(_) => println!("\nНеожиданно: вставка прошла без ошибки — проверьте триггер trg_check_grade_value."),
                Err(error) => println!("\nОшибка триггера (недопустимая оценка):\n{error}"),
            }
            Ok(())
        }
        "13" => {
            let grade_id = selectors::any_grade_row(client)?;
            let wrong_teacher_id = selectors::any_teacher(client)?;
            match client.execute(queries::T15_UPDATE_WRONG_TEACHER, &[&grade_id, &wrong_teacher_id]) {
                Ok(_) => println!("\nНеожиданно: обновление прошло без ошибки — проверьте, что преподаватель действительно не назначен на этот предмет."),
                Err(error) => println!("\nОжидаемая ошибка триггера (преподаватель не назначен на предмет):\n{error}"),
            }
            Ok(())
        }
        "14" => {
            let grade_id = selectors::any_grade_row(client)?;
            client.execute(queries::T15_UPDATE_CORRECT_TEACHER, &[&grade_id])?;
            println!("\nОценка обновлена корректным (назначенным) преподавателем — триггер пропустил изменение.");
            Ok(())
        }

        // ---------------- Задание 3: доп. параметры групп ----------------
        "15" => {
            let group_id = selectors::group(client)?;
            let param_name = selectors::numeric_param_name(client)?;
            show(client, queries::P12_NUMERIC_PARAM_STATS, &[&group_id, &param_name])
        }
        "16" => {
            let search_value = selectors::read_text("Введите искомое значение (подстроку): ")?;
            show(client, queries::P13_SEARCH_TEXT_PARAM, &[&search_value])
        }

        // ---------------- Демонстрация сломанного триггера ----------------
        "99" => run_broken_trigger_demo(client),

        _ => {
            println!("Неизвестная задача.");
            Ok(())
        }
    }
}

/// Демонстрация намеренно сломанного триггера: включаем триггер, который при
/// каждом UPDATE строки grades сам вызывает UPDATE той же строки без условия
/// останова. PostgreSQL прерывает рекурсию ошибкой "stack depth limit exceeded"
/// (или закрывает соединение). После демонстрации триггер обязательно
/// выключается, чтобы не мешать дальнейшей работе клиента.
fn run_broken_trigger_demo(client: &mut Client) -> Result<()> {
    println!("\n!!! ВНИМАНИЕ: сейчас будет включён триггер с самозацикливанием (infinite recursion).");
    println!("Ожидаемый результат — ошибка PostgreSQL 'stack depth limit exceeded' или обрыв соединения.");
    let confirm = selectors::read_line("Продолжить? (yes/no): ")?;
    if confirm.to_lowercase() != "yes" {
        println!("Отменено.");
        return Ok(());
    }

    let grade_id = selectors::any_grade_row(client)?;

    if let Err(error) = client.execute(queries::B16_ENABLE_BROKEN_TRIGGER, &[]) {
        println!("Не удалось включить демо-триггер: {error}");
        return Ok(());
    }
    println!("Триггер trg_broken_self_loop включён. Выполняем UPDATE...");

    let update_result = client.execute(queries::B16_TRIGGER_UPDATE, &[&grade_id]);

    match update_result {
        Ok(_) => println!("\nНеожиданно: UPDATE прошёл без ошибки. Возможно, стек рекурсии в вашей сборке PostgreSQL больше стандартного."),
        Err(error) => println!("\nОжидаемая ошибка самозацикливания триггера:\n{error}"),
    }

    // Соединение после stack depth limit exceeded обычно остаётся рабочим
    // (в отличие от полного краха процесса backend), поэтому пробуем
    // аккуратно выключить триггер обратно. Если соединение всё же разорвано,
    // выведется отдельная ошибка — тогда триггер нужно выключить вручную
    // новым подключением: SELECT faculty.disable_broken_trigger();
    match client.execute(queries::B16_DISABLE_BROKEN_TRIGGER, &[]) {
        Ok(_) => println!("Триггер trg_broken_self_loop выключен, БД восстановлена для дальнейшей работы."),
        Err(error) => println!(
            "Не удалось выключить триггер автоматически ({error}). \
             Подключитесь заново и выполните: SELECT faculty.disable_broken_trigger();"
        ),
    }

    Ok(())
}
