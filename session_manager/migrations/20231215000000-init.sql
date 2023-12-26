CREATE TABLE sessions {
    id bigint NOT NULL,
    application CHAR(128) NOT NULL,
    slots bigint NOT NULL,

    common_data BLOB,

    creation_time: DateTime<Utc>,
    completion_time: Option<DateTime<Utc>>,

    status int NOT NULL,
};

CREATE TABLE tasks {
    id bigint NOT NULL,
    ssn_id bigint NOT NULL,
    input ,
    output ,

     creation_time: DateTime<Utc>,
     completion_time: Option<DateTime<Utc>>,

     state int NOT NULL,
};

CREATE TABLE applications {
    name CHAR(128) NOT NULL,
    shim CHAR(16) NOT NULL,
    command VARCHAR(1024) NOT NULL,

    arguments: Vec<String>,
    environments: Vec<String>,
    
    working_directory VARCHAR(1024),
};