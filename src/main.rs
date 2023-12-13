use rusqlite::params;
use rusqlite::{Connection, Result};
use uuid::Uuid;
use std::{io, env};
use chrono::prelude::*;
use regex::Regex;

#[derive(Debug)]
struct Task {
    id: String,
    descripcion: String,
    time: String,
    task_url: String,
    create_datetime: Option<String>
}

fn config_connection() -> Connection {
    let current_path_exe = env::current_exe().expect("Error to read app path");
    let current_directory = current_path_exe.parent().expect("Errorto read current directoy");
    let exe_path_str = current_directory.to_string_lossy().replace("\\", "/");

    let path = format!("{}/{}", exe_path_str, "task.db");

    let connection = Connection::open(path).expect("Error al conectar con la base de datos");

    connection.execute("CREATE TABLE IF NOT EXISTS TASK (
                id TEXT PRIMARY KEY UNIQUE,
                descripcion TEXT NOT NULL,
                time TEXT NOT NULL,
                task_url TEXT NOT NULL,
                create_datetime DATETIME DEFAULT CURRENT_TIMESTAMP
            );",
            [],).expect("Error al ejecutar la consulta create SQL");

    return connection
}


fn request_data() -> Task {
    let mut description = String::new();
    let mut time = String::new();
    let mut task_url = String::new();
    

    loop {
        println!("Ingrese la descripcion de lo realizado");
        io::stdin().read_line(&mut description).expect("Failed to read line");
        if description.trim() != "" {
            break;
        }
        println!("Ingrese la descripcion de lo realizado")
    }

    loop {
        println!("Ingrese el tiempo dedicado");
        io::stdin().read_line(&mut time).expect("Failed to read line");
        if time.trim() != "" {
            break;
        }
        println!("Ingrese el tiempo dedicado")
    }

    loop {
        println!("Ingrese la url de la tarea");
        io::stdin().read_line(&mut task_url).expect("Failed to read line");
        if task_url.trim() != "" {
            break;
        }
        println!("Ingrese la url de la tarea");
    }

    let new_task = Task{
        id: String::from(Uuid::new_v4()),
        descripcion: String::from(description),
        time: String::from(time),
        task_url: String::from(task_url),
        create_datetime: None
    };

    return new_task;
}

fn create_new_task(connection: Connection) {
    let new_task = request_data();

    connection.execute(
        "INSERT INTO TASK (id, descripcion, time, task_url) VALUES (?, ?, ?, ?)",
        params![new_task.id, new_task.descripcion, new_task.time, new_task.task_url],
    ).expect("Error on execute insert to task table");
}

fn convertir_a_utc0(fecha: Option<DateTime<FixedOffset>>) -> Option<DateTime<Utc>> {
    fecha.and_then(|f| Some(f.with_timezone(&Utc)))
}

fn convertir_a_zona_horaria(fecha_utc: &str, zona_horaria: &str) -> Result<String, &'static str> {
    // Parsear la fecha en formato UTC
    let fecha_utc = DateTime::parse_from_str(format!("{} {}",fecha_utc, "+00:00").as_str(), "%Y-%m-%d %H:%M:%S %z")
        .map_err(|_| "Error al parsear la fecha en formato UTC")?
        .with_timezone(&Utc);

    // Obtener el desplazamiento de la zona horaria
    let desplazamiento = zona_horaria.replace("+", "").replace(":00", "").parse::<i32>().map_err(|_| "Error al parsear el desplazamiento de la zona horaria")?;
    
    // Crear el objeto FixedOffset con el desplazamiento
    let zona_horaria_obj = FixedOffset::east_opt(desplazamiento * 3600).expect("Error to setup timezone");

    // Convertir la fecha a la zona horaria especificada
    let fecha_zona_horaria = fecha_utc.with_timezone(&zona_horaria_obj);

    // Formatear la fecha en la nueva zona horaria
    let fecha_formateada = fecha_zona_horaria.format("%Y-%m-%d %H:%M:%S").to_string();

    Ok(fecha_formateada)
}

fn read_task_by_date(connection: Connection, date: &str, datetime: &str) {
    let start_date_str = format!("{} {} {}", date, "00:00:00", datetime);
    let end_date_str = format!("{} {} {}", date, "23:59:00", datetime);

    // Convertir las cadenas a objetos DateTime<FixedOffset> en UTC-5
    let start_date_utc5 = DateTime::parse_from_str(start_date_str.as_str(), "%Y-%m-%d %H:%M:%S %z")
        .expect("Error parsing start date");
        
    let end_date_utc5 = DateTime::parse_from_str(end_date_str.as_str(), "%Y-%m-%d %H:%M:%S %z")
        .expect("Error parsing end date");

    // Convertir las fechas a UTC+0
    let start_date_utc0 = convertir_a_utc0(Some(start_date_utc5)).expect("Error to convert start date to utc0");
    let end_date_utc0 = convertir_a_utc0(Some(end_date_utc5)).expect("Error to convert end date to utc0");

    let mut stmt = connection
        .prepare("SELECT id, descripcion, time, task_url, create_datetime FROM TASK WHERE create_datetime BETWEEN ? AND ?")
        .expect("Error on request data with select query");


    let task_iter = stmt.query_map(&[&start_date_utc0.format("%Y-%m-%d %H:%M:%S").to_string(),
                                    &end_date_utc0.format("%Y-%m-%d %H:%M:%S").to_string()], |row| {
        Ok(Task {
            id: row.get(0)?,
            descripcion: row.get(1)?,
            time: row.get(2)?,
            task_url: row.get(3)?,
            create_datetime: row.get(4)?
        })
    }).expect("Error mapping data to task entity");

    for task in task_iter {
        let task_result = task.unwrap();
        println!("=========================================");
        println!("ID: {:?}", task_result.id);
        println!("Description: {:?}", task_result.descripcion);
        println!("Time : {:?}", task_result.time);
        println!("Task url: {:?}", task_result.task_url);
        let date_with_timezone = convertir_a_zona_horaria(task_result.create_datetime.expect("Erro to get create_datetime").as_str(), datetime);
        println!("Create Date: {:?}", date_with_timezone.unwrap());
    }
}

fn main() -> Result<()> {

    let args: Vec<String> = env::args().collect();

    let (action, _complement) = get_command(&args);

    match action {
        "new" => {
            let connection = config_connection();
            create_new_task(connection)
        },
        "list" => {
            let connection = config_connection();

            let today = Local::now();
            let mut formatted_date = today.format("%Y-%m-%d").to_string();

            if _complement != "" {
                if is_valid_date(_complement.to_string()) {
                    formatted_date = _complement.to_owned()
                }
            }

            println!("Data from: {}", today.format("%Y-%m-%d %H:%M:%S %Z").to_string());
            read_task_by_date(connection, formatted_date.as_str(), today.format("%Z").to_string().as_str())
        },
        _ => println!("Unhandle command")
    }

    Ok(())
}

fn is_valid_date(date: String) -> bool {
    let re = Regex::new(r"^\d{4}-\d{2}-\d{2}$").unwrap();
    return re.is_match(date.as_str()) 
}

fn get_command(args: &[String]) -> (&str, &str) {
    match args {
        [_x, _y, _z] => {
            return (args[1].as_str(), args[2].as_str())
        },
        [_x, _y] => {
            return (args[1].as_str(), "")
        }
        _ => {
            return ("", "")
        }
    }
}
