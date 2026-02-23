# Table Text Wrapping Test Cases

## Basic Wrapping (2 columns)

| Feature | Description |
|---------|-------------|
| Word Wrap | Long text in table cells should wrap gracefully within the column boundary instead of extending horizontally and forcing the user to scroll to read the content. |
| Short | OK |

## Mixed Short and Long Content

| Name | Role | Notes |
|------|------|-------|
| Alice | Dev | Responsible for the backend API and database migrations. |
| Bob | QA | Handles integration testing, performance benchmarks, and writing end-to-end test suites for all major user flows in the application. |
| Carol | PM | Project management, stakeholder communication, sprint planning, retrospectives, and coordinating between the frontend and backend teams to ensure timely delivery. |

## Many Columns with Long Text

| ID | Title | Author | Status | Description | Tags |
|----|-------|--------|--------|-------------|------|
| 1 | Implement user authentication with OAuth2 and JWT tokens | Alice | In Progress | This task covers the full implementation of user authentication including login, registration, password reset, and session management using JSON Web Tokens. | auth, security, backend |
| 2 | Redesign dashboard layout | Bob | Pending | Complete overhaul of the main dashboard to improve data visualization, add customizable widgets, and implement responsive layout for mobile devices. | ui, frontend, design |
| 3 | Database migration to PostgreSQL | Carol | Done | Migrate all existing data from SQLite to PostgreSQL, update ORM configurations, write migration scripts, and verify data integrity across all tables. | database, infrastructure, migration |

## Single Column Long Text

| Details |
|---------|
| This is a single-column table with a very long paragraph of text that should demonstrate how wrapping works when there is only one column available, giving the text the maximum possible width within the table boundaries. The text should flow naturally across multiple lines. |

## Narrow Columns with Wrapping

| A                                      | B                      | C                          | D                           | E                                      | F                               | G                               | H                                 |
|--------------------------------------|----------------------|--------------------------|---------------------------|--------------------------------------|-------------------------------|-------------------------------|---------------------------------|
| Column one text                        | Column two text        | Column three text          | Column four text            | Column five text                       | Column six text                 | Column seven text               | Column eight text                 |
| More content here that is a bit longer | And some more here too | Yet another cell with text | Still going with more words | Five is the magic number for this cell | Six cells deep in the table now | Lucky number seven cell content | Eight columns of wrapped goodness |
|                                        |                        |                            |                             |                                        |                                 |                                 |                                   |

## Table with Code and URLs

| Resource | URL | Description |
|----------|-----|-------------|
| Rust Book | https://doc.rust-lang.org/book/ch01-00-getting-started.html | The official Rust programming language book, covering everything from installation to advanced topics like lifetimes and trait objects. |
| egui Docs | https://docs.rs/egui/latest/egui/index.html | Documentation for the egui immediate mode GUI library used in this project for all user interface rendering. |
| Comrak | https://github.com/kivikakk/comrak | A CommonMark + GFM compatible Markdown parser and renderer written in Rust, used for parsing markdown content. |

## Alignment with Long Text

| Left-aligned | Center-aligned | Right-aligned |
|:-------------|:--------------:|--------------:|
| This text is left-aligned and should wrap at the column boundary while maintaining its left alignment throughout all wrapped lines. | This centered text should wrap while keeping the center alignment, which can look interesting with longer content spanning multiple lines. | Right-aligned text wrapping should keep content flush to the right edge of the cell, even when the text spans multiple visual lines. |

## Edge Cases

| Header | Content |
|--------|---------|
|  | Empty cell above, this cell has content |
| Tiny | x |
| A cell with a single very long word: Supercalifragilisticexpialidocious | Normal text |
| Normal | A cell with special characters: <>&"' and unicode: äöü ñ 日本語 한국어 中文 |

## Real-World Data Table

| Country | Capital | Population | GDP (Trillion USD) | Official Language | Currency | Notable Feature |
|---------|---------|------------|-------------------|-------------------|----------|-----------------|
| United States | Washington D.C. | 331,002,651 | 25.46 | English | US Dollar (USD) | World's largest economy by nominal GDP, diverse geography spanning from arctic to tropical climates |
| China | Beijing | 1,411,778,724 | 17.96 | Mandarin Chinese | Renminbi (CNY) | World's most populous country, rapid economic growth over the past four decades transforming it into a global superpower |
| Germany | Berlin | 83,783,942 | 4.07 | German | Euro (EUR) | Europe's largest economy, known for engineering excellence and the automotive industry including BMW, Mercedes-Benz, and Volkswagen |
| Japan | Tokyo | 125,681,593 | 4.23 | Japanese | Yen (JPY) | Third-largest economy globally, known for technological innovation, cultural exports including anime and manga, and the highest life expectancy |
| Brazil | Brasília | 212,559,417 | 1.92 | Portuguese | Real (BRL) | Largest country in South America, home to the Amazon rainforest which produces approximately 20% of the world's oxygen |