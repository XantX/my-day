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
    create_date: Option<String>
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
                create_date DATE DEFAULT CURRENT_DATE
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
            println!("La descripcion de la tarea es: {}", description);
            break;
        }
        println!("Ingrese un valor para la descripcion de la tarea")
    }

    loop {
        println!("Ingrese el tiempo dedicado");
        io::stdin().read_line(&mut time).expect("Failed to read line");
        if time.trim() != "" {
            println!("El tiempo dedicado es de {}", time);
            break;
        }
        println!("Ingrese un valor para el tiempo dedicado")
    }

    loop {
        println!("Ingrese la url de la tarea");
        io::stdin().read_line(&mut task_url).expect("Failed to read line");
        if task_url.trim() != "" {
            println!("La url escrita es {}", task_url);
            break;
        }
        println!("Ingrese un valor para la url de la tarea");
    }

    let new_task = Task{
        id: String::from(Uuid::new_v4()),
        descripcion: String::from(description),
        time: String::from(time),
        task_url: String::from(task_url),
        create_date: None
    };

    return new_task;
}

fn create_new_task(connection: Connection) {
    let new_task = request_data();

    connection.execute(
        "INSERT INTO TASK (id, descripcion, time, task_url) VALUES (?, ?, ?, ?)",
        params![new_task.id, new_task.descripcion, new_task.time, new_task.task_url],
    ).expect("Erro on execute insert to task table");
}

fn read_task_by_date(connection: Connection, date: &str) {
    let mut stmt = connection.prepare("SELECT id, descripcion, time, task_url, create_date FROM TASK WHERE create_date = ?").expect("Error on request data with select query");

    let task_iter = stmt.query_map(&[&date], |row| {
        Ok(Task {
            id: row.get(0)?,
            descripcion: row.get(1)?,
            time: row.get(2)?,
            task_url: row.get(3)?,
            create_date: row.get(4)?
        })
    }).expect("Error mapping data to task entity");

    println!("Data from {}", date);
    for task in task_iter {
        let task_result = task.unwrap();
        println!("=========================================");
        println!("ID: {:?}", task_result.id);
        println!("Description: {:?}", task_result.descripcion);
        println!("Time : {:?}", task_result.time);
        println!("Task url: {:?}", task_result.task_url);
        println!("Create Date: {:?}", task_result.create_date);
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
            read_task_by_date(connection, formatted_date.as_str())
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
