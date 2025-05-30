use actix_web::{get, http::header, middleware::Logger, post, web, put, App, HttpResponse, HttpServer, Responder, Result};
use serde::{Deserialize, Serialize};
use actix_cors::Cors;
use rusqlite::{Connection, params, Result as RusqliteResult, Transaction};
use rand::seq::SliceRandom;
use rand::thread_rng;

// --- Struct Definitions ---
#[derive(Deserialize, Serialize)]
struct TaLogin {
    gmail: String,
}

#[derive(Serialize, Default)]
pub struct WeeklyInfo {
    pub name: String,
    pub group_id: String,
    pub ta: Option<String>,
    pub attendance: Option<String>,
    pub fa: Option<f64>,
    pub fb: Option<f64>,
    pub fc: Option<f64>,
    pub fd: Option<f64>,
    pub bonus_attendance: Option<String>,
    pub bonus_answer_quality: Option<String>,
    pub bonus_follow_up: Option<String>,
    pub exercise_submitted: Option<String>,
    pub exercise_test_passing: Option<String>,
    pub exercise_good_documentation: Option<String>,
    pub exercise_good_structure: Option<String>,
    pub total: Option<f64>,
    pub mail: String,
    pub week: i32,
}



#[derive(Serialize)]
struct TAData {
    id: i32,
    name: String,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct StudentEntry {
    pub name: String,
    pub mail: String,
    pub attendance: Option<String>,
    pub week: i32, // For add_students, this is primary. For update_weekly_data, path param is authoritative.
    pub group_id: String,
    pub ta: Option<String>,
    pub fa: Option<f64>,
    pub fb: Option<f64>,
    pub fc: Option<f64>,
    pub fd: Option<f64>,
    pub bonus_attendance: Option<String>,
    pub bonus_answer_quality: Option<String>,
    pub bonus_follow_up: Option<String>,
    pub exercise_submitted: Option<String>,
    pub exercise_test_passing: Option<String>,
    pub exercise_good_documentation: Option<String>,
    pub exercise_good_structure: Option<String>,
    pub total: Option<f64>,
}


// --- Handlers ---
#[post("/login")]
async fn login(item: web::Json<TaLogin>) -> impl Responder {
    let ta_list: Vec<&str> = vec!["tusharvyas316@gmail.com", "raj@bitshala.org" , "setu@bitshala.org"
,"anmolsharma0234@gmail.com"
,"balajic86@gmail.com"
,"delcinraj@gmail.com"
,"beulahebenezer777@gmail.com" ];
    if ta_list.iter().any(|ta_gmail| ta_gmail == &item.gmail) {
        HttpResponse::Ok().json(TaLogin { gmail: format!("Access granted for: {}", item.gmail) })
    } else {
        HttpResponse::Unauthorized().json(serde_json::json!({
            "status": "error",
            "message": format!("Access denied for: {}", item.gmail)
        }))
    }
}

#[get("/students/count")]
async fn get_total_student_count() -> impl Responder {
    match Connection::open("classroom.db") {
        Ok(conn) => {
            let count: i64 = conn.query_row(
                "SELECT COUNT(*) FROM students",
                [],
                |row| row.get(0),
            ).unwrap_or(0);
            HttpResponse::Ok().json(serde_json::json!({ "total_students": count }))
        }
        Err(e) => HttpResponse::InternalServerError().body(format!("DB open error: {}", e)),
    }
}

#[get("/attendance/weekly_counts")]
async fn get_weekly_attendance_counts() -> impl Responder {
    match Connection::open("classroom.db") {
        Ok(conn) => {
            let mut stmt = match conn.prepare(
                "SELECT week, COUNT(*) as attended_count FROM students
                 WHERE attendance = 'yes'
                 GROUP BY week ORDER BY week"
            ) {
                Ok(s) => s,
                Err(e) => return HttpResponse::InternalServerError().body(format!("Prepare error: {}", e)),
            };

            let rows_result = stmt.query_map([], |row| {
                Ok(serde_json::json!({
                    "week": row.get::<_, i32>(0)?,
                    "attended": row.get::<_, i64>(1)?,
                }))
            });

            match rows_result {
                Ok(rows) => {
                    let list: Vec<_> = rows.filter_map(Result::ok).collect();
                    HttpResponse::Ok().json(list)
                }
                Err(e) => HttpResponse::InternalServerError().body(format!("Query error: {}", e)),
            }
        }
        Err(e) => HttpResponse::InternalServerError().body(format!("DB open error: {}", e)),
    }
}


#[get("/weekly_data/{week}")]
async fn get_weekly_data_or_common(week: web::Path<i32>) -> impl Responder {
    let week = week.into_inner();
    let conn = match Connection::open("classroom.db") {
        Ok(c) => c,
        Err(e) => return HttpResponse::InternalServerError().body(format!("DB open error: {}", e)),
    };

    // Step 1: Check if week data exists
    let count: i64 = match conn.query_row(
        "SELECT COUNT(*) FROM students WHERE week = ?1",
        [week],
        |row| row.get(0),
    ) {
        Ok(c) => c,
        Err(e) => return HttpResponse::InternalServerError().body(format!("Count query error: {}", e)),
    };

    if count > 0 {
        // Case 1: Full weekly data exists for this week → return it
        let mut stmt = conn.prepare("SELECT * FROM students WHERE week = ?1")
            .map_err(|e| HttpResponse::InternalServerError().body(format!("Prepare weekly data error: {}", e))).unwrap();

        let rows_result = stmt.query_map([week], |row| {
            Ok(WeeklyInfo {
                name: row.get(0)?,
                group_id: row.get(1)?,
                ta: row.get(2).ok(),
                attendance: row.get(3).ok(),
                fa: row.get(4).ok(),
                fb: row.get(5).ok(),
                fc: row.get(6).ok(),
                fd: row.get(7).ok(),
                bonus_attendance: row.get(8).ok(),
                bonus_answer_quality: row.get(9).ok(),
                bonus_follow_up: row.get(10).ok(),
                exercise_submitted: row.get(11).ok(),
                exercise_test_passing: row.get(12).ok(),
                exercise_good_documentation: row.get(13).ok(),
                exercise_good_structure: row.get(14).ok(),
                total: row.get(15).ok(),
                mail: row.get(16)?,
                week: row.get(17)?,
            })
        });

        match rows_result {
            Ok(rows) => {
                let list: Vec<WeeklyInfo> = rows.filter_map(Result::ok).collect();
                HttpResponse::Ok().json(list)
            }
            Err(e) => HttpResponse::InternalServerError().body(format!("Query map weekly data error: {}", e)),
        }
    } else if week >= 2 {
    let prev_week = week - 1;
    let mut stmt = match conn.prepare(
        &format!(
            "SELECT * FROM students
                WHERE week = {}
                ORDER BY CASE attendance
                    WHEN 'yes' THEN 0
                    WHEN 'no' THEN 1
                    ELSE 2
                END;",
            prev_week
        )
    ) {
        Ok(s) => s,
        Err(e) => return HttpResponse::InternalServerError().body(format!("Prepare previous week data error: {}", e)),
    };

    let rows_result = stmt.query_map([], |row| {

        let attendance: Option<String> = row.get(3)?;
             let mut rng = thread_rng();
             let present_tas = vec!["Anmol Sharma", "Bala", "delcin", "Beulah Evanjalin", "Raj"];

            // Assign groups 1-4 in round-robin for present students, group 5 for absent.
            // Use a static counter outside the closure to persist group assignment across rows.
            thread_local! {
                static GROUP_COUNTER: std::cell::RefCell<usize> = std::cell::RefCell::new(1);
            }

            // Assign groups 1-4 in round-robin for present students, and assign each group to a specific TA.
            // Group 1: Anmol Sharma, Group 2: Bala, Group 3: delcin, Group 4: Beulah Evanjalin, Group 5: Saurabh (absent)
            let group_ta_map = [
                ("Group 1", "Anmol Sharma"),
                ("Group 2", "Bala"),
                ("Group 3", "delcin"),
                ("Group 4", "Beulah Evanjalin"),
            ];

            let (reassigned_group, assigned_ta) = match attendance.as_deref() {
                Some("yes") => {
                    // Use thread_local counter for round-robin group assignment
                    let group_idx = GROUP_COUNTER.with(|counter| {
                        let mut val = counter.borrow_mut();
                        let idx = *val - 1;
                        *val = if *val == 4 { 1 } else { *val + 1 };
                        idx
                    });
                    let (group, ta) = group_ta_map[group_idx];
                    (group.to_string(), ta.to_string())
                },
                _ => ("Group 5 (Absent)".to_string(), "Saurabh".to_string()),
            };

        Ok(WeeklyInfo {
            name: row.get(0)?,
            mail: row.get(16)?,
            group_id: reassigned_group,
            ta: Some(assigned_ta),
            total: Some(row.get::<_, f64>(4).unwrap_or(0.0)),
            week: week,
            ..Default::default()
        })
    });

    match rows_result {
        Ok(rows) => {
            let list: Vec<WeeklyInfo> = rows.filter_map(Result::ok).collect();
            HttpResponse::Ok().json(list)
        }
        Err(e) => HttpResponse::InternalServerError().body(format!("Query map from week 1 fallback error: {}", e)),
    }
} else {
        // Case 3: Fallback to all students when no prior or current week data (e.g. week 1)
        let mut stmt = conn.prepare("SELECT * FROM students
                    WHERE week = 0
                    ORDER BY CASE attendance
                        WHEN 'yes' THEN 0
                        WHEN 'no' THEN 1
                        ELSE 2
                    END;
                ")
                .map_err(|e| HttpResponse::InternalServerError().body(format!("Prepare common info error: {}", e))).unwrap();

        let rows_result = stmt.query_map([], |row| {
             let attendance: Option<String> = row.get(3)?;
             let mut rng = thread_rng();
             let present_tas = vec!["Anmol Sharma", "Bala", "delcin", "Beulah Evanjalin", "Raj"];


            thread_local! {
                static SHUFFLED_TAS: std::cell::RefCell<Option<Vec<String>>> = std::cell::RefCell::new(None);
                static GROUP_COUNTER: std::cell::RefCell<usize> = std::cell::RefCell::new(1);
            }

            let (reassigned_group, assigned_ta) = match attendance.as_deref() {
                Some("yes") => {
                    // Shuffle TAs if not already shuffled for this request
                    let shuffled_tas = SHUFFLED_TAS.with(|cell| {
                        let mut opt = cell.borrow_mut();
                        if opt.is_none() {
                            let mut tas = present_tas.iter().map(|s| s.to_string()).collect::<Vec<_>>();
                            tas.shuffle(&mut rng);
                            *opt = Some(tas);
                        }
                        opt.as_ref().unwrap().clone()
                    });

                    // Assign group in round-robin and TA by group index
                    let group_idx = GROUP_COUNTER.with(|counter| {
                        let mut val = counter.borrow_mut();
                        let idx = *val - 1;
                        *val = if *val == 4 { 1 } else { *val + 1 };
                        idx
                    });
                    let group = format!("Group {}", group_idx + 1);
                    let ta = shuffled_tas.get(group_idx).cloned().unwrap_or_else(|| "Raj".to_string());
                    (group, ta)
                },
                _ => ("Group 5 (Absent)".to_string(), "Saurabh".to_string()),
            };

            Ok(WeeklyInfo {
                name: row.get(0)?,
                group_id: reassigned_group,
                ta: Some(assigned_ta),
                attendance: row.get(13).ok(),
                fa: row.get(4).ok(),
                fb: row.get(5).ok(),
                fc: row.get(6).ok(),
                fd: row.get(7).ok(),
                bonus_attendance: row.get(8).ok(),
                bonus_answer_quality: row.get(9).ok(),
                bonus_follow_up: row.get(10).ok(),
                exercise_submitted: row.get(11).ok(),
                exercise_test_passing: row.get(12).ok(),
                exercise_good_documentation: row.get(13).ok(),
                exercise_good_structure: row.get(14).ok(),
                total: row.get(15).ok(),
                mail: row.get(16)?,
                week: row.get(17)?,
            })
        });

        match rows_result {
            Ok(rows) => {
                let list: Vec<WeeklyInfo> = rows.filter_map(Result::ok).collect();
                HttpResponse::Ok().json(list)
            }
            Err(e) => HttpResponse::InternalServerError().body(format!("Query map common info error: {}", e)),
        }
    }
}


#[post("/weekly_data/{week}")]
async fn add_weekly_data(
    week: web::Path<i32>,
    student_data: web::Json<Vec<StudentEntry>>,
) -> Result<HttpResponse, actix_web::Error> {
    let week_number = week.into_inner();

    let mut conn = Connection::open("classroom.db")
        .map_err(|e| actix_web::error::ErrorInternalServerError(e.to_string()))?;

    let tx = conn
        .transaction()
        .map_err(|e| actix_web::error::ErrorInternalServerError(e.to_string()))?;

    for entry in student_data.into_inner() {
        // Check if the row exists
        let exists: i64 = tx
            .query_row(
                "SELECT COUNT(*) FROM students WHERE week = ?1 AND mail = ?2",
                params![week_number, entry.mail],
                |row| row.get(0),
            )
            .unwrap_or(0);

        if exists > 0 {
            // UPDATE if exists
            tx.execute(
                r#"
                UPDATE students SET
                    name = ?2,
                    attendance = ?3,
                    group_id = ?4,
                    ta = ?5,
                    fa = ?6,
                    fb = ?7,
                    fc = ?8,
                    fd = ?9,
                    bonus_attendance = ?10,
                    bonus_answer_quality = ?11,
                    bonus_follow_up = ?12,
                    exercise_submitted = ?13,
                    exercise_test_passing = ?14,
                    exercise_good_documentation = ?15,
                    exercise_good_structure = ?16,
                    total = ?17
                WHERE week = ?1 AND mail = ?18
                "#,
                params![
                    week_number,
                    entry.name,
                    entry.attendance,
                    entry.group_id,
                    entry.ta,
                    entry.fa,
                    entry.fb,
                    entry.fc,
                    entry.fd,
                    entry.bonus_attendance,
                    entry.bonus_answer_quality,
                    entry.bonus_follow_up,
                    entry.exercise_submitted,
                    entry.exercise_test_passing,
                    entry.exercise_good_documentation,
                    entry.exercise_good_structure,
                    entry.total,
                    entry.mail,
                ],
            ).map_err(|e| actix_web::error::ErrorInternalServerError(e.to_string()))?;
        } else {
            // INSERT if not exists
            tx.execute(
                r#"
                INSERT INTO students (
                    week, name, mail, attendance, group_id, ta,
                    fa, fb, fc, fd,
                    bonus_attendance, bonus_answer_quality, bonus_follow_up,
                    exercise_submitted, exercise_test_passing,
                    exercise_good_documentation, exercise_good_structure, total
                ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16, ?17, ?18)
                "#,
                params![
                    week_number,
                    entry.name,
                    entry.mail,
                    entry.attendance,
                    entry.group_id,
                    entry.ta,
                    entry.fa,
                    entry.fb,
                    entry.fc,
                    entry.fd,
                    entry.bonus_attendance,
                    entry.bonus_answer_quality,
                    entry.bonus_follow_up,
                    entry.exercise_submitted,
                    entry.exercise_test_passing,
                    entry.exercise_good_documentation,
                    entry.exercise_good_structure,
                    entry.total,
                ],
            ).map_err(|e| actix_web::error::ErrorInternalServerError(e.to_string()))?;
        }
    }

    tx.commit()
        .map_err(|e| actix_web::error::ErrorInternalServerError(format!("Commit failed: {}", e)))?;

    Ok(HttpResponse::Ok().body("Weekly data inserted/updated successfully"))
}


#[get("/tas")]
async fn get_tas() -> impl Responder {
     match Connection::open("classroom.db") {
        Ok(conn) => {
            let mut stmt = match conn.prepare("SELECT id, name FROM ta") {
                Ok(s) => s,
                Err(e) => return HttpResponse::InternalServerError().body(format!("Prepare TA list error: {}", e)),
            };
            let rows_result = stmt.query_map([], |row| Ok(TAData { id: row.get(0)?, name: row.get(1)? }));
            match rows_result {
                Ok(rows) => {
                    let list: Vec<TAData> = rows.filter_map(Result::ok).collect();
                    HttpResponse::Ok().json(list)
                }
                Err(e) => HttpResponse::InternalServerError().body(format!("Query map TA list error: {}", e)),
            }
        }
        Err(e) => HttpResponse::InternalServerError().body(format!("DB open error for TA list: {}", e)),
    }
}



#[actix_web::main]
async fn main() -> std::io::Result<()> {
    env_logger::init_from_env(env_logger::Env::default().default_filter_or("info"));

    HttpServer::new(|| {
        let cors = Cors::default()
            .allowed_origin("http://localhost:5173")
            .allowed_methods(vec!["GET", "POST", "PUT", "DELETE"])
            .allowed_headers(vec![header::AUTHORIZATION, header::ACCEPT, header::CONTENT_TYPE])
            .supports_credentials()
            .max_age(3600);

        App::new()
            .wrap(cors)
            .wrap(Logger::default())
            .service(login)
            .service(get_tas)
            .service(get_weekly_data_or_common)
            .service(add_weekly_data)
            .service(get_total_student_count)
            .service(get_weekly_attendance_counts)
        
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}