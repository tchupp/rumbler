use crate::error::RumblerError;

pub fn render(sql: &str) -> Result<String, RumblerError> {
    subst::substitute(sql, &subst::Env).map_err(|e| RumblerError::Template(e.to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use sealed_test::prelude::*;

    #[test]
    fn test_no_placeholders() {
        let sql = "CREATE TABLE users (id SERIAL PRIMARY KEY);";
        assert_eq!(render(sql).unwrap(), sql);
    }

    #[sealed_test(env = [("FOO", "bar")])]
    fn test_single_placeholder() {
        assert_eq!(render("CREATE TABLE ${FOO};").unwrap(), "CREATE TABLE bar;");
    }

    #[sealed_test(env = [("FOO", "bar")])]
    fn test_short_form() {
        assert_eq!(render("CREATE TABLE $FOO;").unwrap(), "CREATE TABLE bar;");
    }

    #[sealed_test(env = [("DB", "mydb"), ("SCHEMA", "public")])]
    fn test_multiple_placeholders() {
        assert_eq!(render("USE ${DB}.${SCHEMA};").unwrap(), "USE mydb.public;");
    }

    #[sealed_test(env = [("T", "users")])]
    fn test_repeated_placeholder() {
        assert_eq!(
            render("DROP ${T}; CREATE ${T};").unwrap(),
            "DROP users; CREATE users;"
        );
    }

    #[test]
    fn test_missing_env_var() {
        let result = render("${NONEXISTENT_VAR_12345}");
        assert!(result.is_err());
    }

    #[sealed_test(env = [("TABLE_NAME", "users")])]
    fn test_default_value() {
        assert_eq!(
            render("CREATE TABLE ${TABLE_NAME:fallback};").unwrap(),
            "CREATE TABLE users;"
        );
        assert_eq!(
            render("CREATE TABLE ${MISSING:fallback};").unwrap(),
            "CREATE TABLE fallback;"
        );
    }

    // --- PostgreSQL compatibility tests ---
    // These tests document known conflicts between subst's syntax and PostgreSQL SQL.

    #[test]
    #[should_panic(expected = "Invalid escape sequence")]
    fn test_pg_escape_string_breaks() {
        // PostgreSQL E-string syntax uses backslash escapes: E'\n', E'\t'
        // subst treats \ as an escape character and only allows \$ \{ \} \: \\
        render(r"INSERT INTO t VALUES (E'\n')").unwrap();
    }

    #[test]
    #[should_panic(expected = "Missing variable name")]
    fn test_pg_dollar_quoting_breaks() {
        // PostgreSQL dollar-quoted strings: $$ or $tag$
        // subst parses $$ as a missing variable name
        render("CREATE FUNCTION f() RETURNS void AS $$ BEGIN NULL; END; $$ LANGUAGE plpgsql")
            .unwrap();
    }

    #[test]
    #[should_panic(expected = "No such variable")]
    fn test_pg_positional_param_breaks() {
        // PostgreSQL prepared statement parameters: $1, $2, etc.
        // subst parses $1 as variable "1"
        render("PREPARE stmt AS SELECT $1::int").unwrap();
    }

    #[test]
    fn test_pg_single_dollar_at_end() {
        // A bare $ at end of string triggers a "missing variable name" error
        let result = render("SELECT 'cost: $'");
        assert!(result.is_err());
    }

    #[test]
    fn test_pg_regex_backslash_breaks() {
        // PostgreSQL regex patterns use backslashes: ~ E'\\d+'
        let result = render(r"SELECT * FROM t WHERE col ~ E'\d+'");
        assert!(result.is_err());
    }
}
