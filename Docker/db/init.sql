CREATE DATABASE IF NOT EXISTS mydatabase;

GRANT ALL PRIVILEGES ON mydatabase.* TO 'myuser' @'%' IDENTIFIED BY 'mypassword';

USE mydatabase;

CREATE TABLE IF NOT EXISTS circles (
    id INT AUTO_INCREMENT PRIMARY KEY,
    name VARCHAR(255) NOT NULL,
    capacity INT NOT NULL,
    owner_id INT NOT NULL
);

CREATE TABLE IF NOT EXISTS members (
    id INT AUTO_INCREMENT PRIMARY KEY,
    name VARCHAR(255) NOT NULL,
    grade INT NOT NULL,
    circle_id INT,
    age INT NOT NULL DEFAULT 20,
    major VARCHAR(255) NOT NULL DEFAULT 'other',
    FOREIGN KEY (circle_id) REFERENCES circles(id) ON DELETE CASCADE
);

INSERT INTO
    circles (name, capacity, owner_id)
VALUES
    ('Circle A', 5, 1),
    ('Circle B', 8, 2),
    ('Circle C', 10, 3);

INSERT INTO
    members (name, grade, circle_id, age, major)
VALUES
    ('Alice', 3, 1, 21, 'math'),
    ('Bob', 2, 2, 22, 'math'),
    ('Charlie', 3, 3, 23, 'math'),
    ('David', 4, 1, 21, 'math'),
    ('Eve', 2, 2, 19, 'math'),
    ('Frank', 4, 3, 20, 'math');