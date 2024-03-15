use libsql::Builder;
use rustler::{Binary, Decoder, Encoder, NifStruct, ResourceArc, Term};

pub mod task;

type Error = String;

#[derive(NifStruct, Clone, Debug)]
#[module = "Libsql.Result"]
struct ResultStruct {
    columns: Option<Vec<String>>,
    last_insert_id: Option<i64>,
    num_rows: Option<usize>,
    rows: Option<Vec<Vec<Value>>>,
}

#[derive(NifStruct, Clone)]
#[module = "Libsql.Database"]
struct Database {
    database: ResourceArc<InnerDatabase>,
}

struct InnerDatabase(libsql::Database);

#[derive(NifStruct, Clone)]
#[module = "Libsql.Connection"]
struct Connection {
    connection: ResourceArc<InnerConnection>,
}

struct InnerConnection(libsql::Connection);

#[derive(Debug, Clone)]
struct Value(libsql::Value);

impl From<Value> for libsql::Value {
    fn from(value: Value) -> Self {
        value.0
    }
}

impl From<libsql::Value> for Value {
    fn from(value: libsql::Value) -> Self {
        Value(value)
    }
}

impl<'a> Decoder<'a> for Value {
    fn decode(term: Term<'a>) -> rustler::NifResult<Self> {
        if let Ok(data) = term.atom_to_string() {
            if data == "nil" {
                return Ok(libsql::Value::Null.into());
            } else {
                return Ok(libsql::Value::Text(data).into());
            }
        }
        if let Ok(data) = term.decode::<i64>() {
            return Ok(libsql::Value::Integer(data).into());
        }
        if let Ok(data) = term.decode::<bool>() {
            if data {
                return Ok(libsql::Value::Integer(1).into());
            } else {
                return Ok(libsql::Value::Integer(0).into());
            }
        }
        if let Ok(data) = term.decode::<f64>() {
            return Ok(libsql::Value::Real(data).into());
        }
        if let Ok(data) = term.decode::<String>() {
            return Ok(libsql::Value::Text(data).into());
        }
        if let Ok(data) = term.decode::<Binary>() {
            return Ok(libsql::Value::Blob(data.as_slice().to_vec()).into());
        }

        return Err(rustler::Error::BadArg);
    }
}

impl Encoder for Value {
    fn encode<'a>(&self, env: rustler::Env<'a>) -> Term<'a> {
        match self.0 {
            libsql::Value::Null => None::<()>.encode(env),
            libsql::Value::Integer(data) => data.encode(env),
            libsql::Value::Real(data) => data.encode(env),
            libsql::Value::Text(ref data) => data.encode(env),
            libsql::Value::Blob(ref data) => data.encode(env),
        }
    }
}

#[rustler::nif]
fn add(a: i64, b: i64) -> i64 {
    a + b
}

#[rustler::nif]
fn new_local(path: &str) -> Result<Database, Error> {
    let libsql_db = task::block_on(Builder::new_local(path).build()).map_err(|e| e.to_string())?;

    Ok(Database {
        database: ResourceArc::new(InnerDatabase(libsql_db)),
    })
}

#[rustler::nif]
fn new_remote(url: String, auth_token: String) -> Result<Database, Error> {
    let libsql_db =
        task::block_on(Builder::new_remote(url, auth_token).build()).map_err(|e| e.to_string())?;

    Ok(Database {
        database: ResourceArc::new(InnerDatabase(libsql_db)),
    })
}

#[rustler::nif]
fn open_db(database: Database) -> Result<Connection, Error> {
    let libsql_conn = database.database.0.connect().map_err(|e| e.to_string())?;
    Ok(Connection {
        connection: ResourceArc::new(InnerConnection(libsql_conn)),
    })
}

#[rustler::nif]
fn query_on_conn<'a>(
    connection: Connection,
    statement: String,
    params: Vec<Value>,
) -> Result<ResultStruct, Error> {
    let res: libsql::Result<_> = task::block_on(async move {
        let conn = connection.connection;
        let mut query_results: libsql::Rows = conn.0.query(&statement, params).await?;

        let column_count: usize = query_results.column_count().try_into().unwrap();

        let mut columns = Vec::with_capacity(column_count);
        for idx in 0..column_count {
            let name = query_results
                .column_name(idx.try_into().unwrap())
                .unwrap_or("");
            columns.push(name.to_string())
        }

        let mut data = Vec::new();
        while let Ok(Some(row)) = query_results.next().await {
            let mut row_data = Vec::with_capacity(columns.len());
            for (idx, _) in columns.iter().enumerate() {
                row_data.push(row.get_value(idx as i32).map(|d| d.into())?)
            }
            data.push(row_data)
        }

        let out = Ok(ResultStruct {
            num_rows: Some(data.len()),
            rows: Some(data),
            columns: Some(columns),
            last_insert_id: Some(conn.0.last_insert_rowid()),
        });
        out
    });

    res.map_err(|e| e.to_string())
}

rustler::init!(
    "Elixir.Libsql.Native",
    [add, new_local, new_remote, open_db, query_on_conn],
    load = load
);

fn load(env: rustler::Env, _: rustler::Term) -> bool {
    rustler::resource!(InnerDatabase, env);
    rustler::resource!(InnerConnection, env);

    true
}
