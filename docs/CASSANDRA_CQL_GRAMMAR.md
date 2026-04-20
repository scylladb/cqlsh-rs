# Apache Cassandra CQL Grammar Rules
## Official Source: Apache Cassandra trunk branch

**Repository**: https://github.com/apache/cassandra  
**Source Files**:
- `pylib/cqlshlib/cql3handling.py` — CQL statement grammar
- `pylib/cqlshlib/cqlshhandling.py` — cqlsh shell command grammar

**Commit SHA**: daadd25fb29ae76110e0764ce568272c7fec5682

---

## CQL Statement Structure

### Start Rule
```bnf
<Start> ::= <CQL_Statement>*
          ;

<CQL_Statement> ::= [statements]=<statementBody> ";"
                  ;

<statementBody> ::= <useStatement>
                  | <selectStatement>
                  | <dataChangeStatement>
                  | <schemaChangeStatement>
                  | <authenticationStatement>
                  | <authorizationStatement>
                  ;

<dataChangeStatement> ::= <insertStatement>
                        | <updateStatement>
                        | <deleteStatement>
                        | <truncateStatement>
                        | <batchStatement>
                        ;

<schemaChangeStatement> ::= <createKeyspaceStatement>
                          | <createColumnFamilyStatement>
                          | <copyTableStatement>
                          | <createIndexStatement>
                          | <createMaterializedViewStatement>
                          | <createUserTypeStatement>
                          | <createFunctionStatement>
                          | <createAggregateStatement>
                          | <createTriggerStatement>
                          | <addIdentityStatement>
                          | <dropKeyspaceStatement>
                          | <dropColumnFamilyStatement>
                          | <dropIndexStatement>
                          | <dropMaterializedViewStatement>
                          | <dropUserTypeStatement>
                          | <dropFunctionStatement>
                          | <dropAggregateStatement>
                          | <dropTriggerStatement>
                          | <dropIdentityStatement>
                          | <alterTableStatement>
                          | <alterKeyspaceStatement>
                          | <alterUserTypeStatement>
                          | <commentOnKeyspaceStatement>
                          | <commentOnTableStatement>
                          | <commentOnColumnStatement>
                          | <commentOnTypeStatement>
                          | <securityLabelOnKeyspaceStatement>
                          | <securityLabelOnTableStatement>
                          | <securityLabelOnColumnStatement>
                          | <securityLabelOnTypeStatement>
                          ;

<authenticationStatement> ::= <createUserStatement>
                            | <alterUserStatement>
                            | <dropUserStatement>
                            | <listUsersStatement>
                            | <createGeneratedRoleStatement>
                            | <createRoleStatement>
                            | <alterRoleStatement>
                            | <dropRoleStatement>
                            | <listRolesStatement>
                            | <listSuperUsersStatement>
                            ;

<authorizationStatement> ::= <grantStatement>
                           | <grantRoleStatement>
                           | <revokeStatement>
                           | <revokeRoleStatement>
                           | <listPermissionsStatement>
                           ;
```

---

## Query Statements

### USE Statement
```bnf
<useStatement> ::= "USE" <keyspaceName>
                 ;
```

### SELECT Statement
```bnf
<selectStatement> ::= "SELECT" ( "JSON" )? <selectClause>
                         "FROM" (cf=<columnFamilyName> | mv=<materializedViewName>)
                           ( "WHERE" <whereClause> )?
                           ( "GROUP" "BY" <groupByClause> ( "," <groupByClause> )* )?
                           ( "ORDER" "BY" <orderByClause> ( "," <orderByClause> )* )?
                           ( "PER" "PARTITION" "LIMIT" perPartitionLimit=<wholenumber> )?
                           ( "LIMIT" limit=<wholenumber> )?
                           ( "ALLOW" "FILTERING" )?
                           ( "WITH" <options> )?
                     ;

<whereClause> ::= <relation> ( "AND" <relation> )*
                ;

<relation> ::= [rel_lhs]=<cident> ( "[" <term> "]" )? ( "=" | "<" | ">" | "<=" | ">=" | "!=" | ( "NOT" )? "CONTAINS" ( "KEY" )? ) (<term> | <operandFunctions>)
             | token="TOKEN" "(" [rel_tokname]=<cident>
                                 ( "," [rel_tokname]=<cident> )*
                             ")" ("=" | "<" | ">" | "<=" | ">=") <tokenDefinition>
             | [rel_lhs]=<cident> (( "NOT" )? "IN" ) "(" <term> ( "," <term> )* ")"
             | [rel_lhs]=<cident> "BETWEEN" <term> "AND" <term>
             | <operandFunctions>
             ;

<selectClause> ::= "DISTINCT"? <selector> ("AS" <cident>)? ("," <selector> ("AS" <cident>)?)*
                 | "*"
                 ;

<selector> ::= [colname]=<cident> ( "[" ( <term> ( ".." <term> "]" )? | <term> ".." ) )?
             | <udtSubfieldSelection>
             | "CAST" "(" <selector> "AS" <storageType> ")"
             | "TTL" "(" [colname]=<cident> ")"
             | "TOKEN" "(" [colname]=<cident> ")"
             | <aggregateMathFunctions>
             | <scalarMathFunctions>
             | <collectionFunctions>
             | <currentTimeFunctions>
             | <maskFunctions>
             | <timeConversionFunctions>
             | <writetimeFunctions>
             | <functionName> <selectionFunctionArguments>
             | <term>
             ;

<orderByClause> ::= [ordercol]=<cident> ( "ASC" | "DESC" )?
                  ;

<groupByClause> ::= [groupcol]=<cident>
                  | <functionName><groupByFunctionArguments>
                  ;
```

---

## Data Modification Statements

### INSERT Statement
```bnf
<insertStatement> ::= "INSERT" "INTO" cf=<columnFamilyName>
                      ( ( "(" [colname]=<cident> ( "," [colname]=<cident> )* ")"
                          "VALUES" "(" [newval]=<term> ( valcomma="," [newval]=<term> )* valcomma=")")
                        | ("JSON" <stringLiteral>))
                      ( "IF" "NOT" "EXISTS")?
                      ( "USING" [insertopt]=<usingOption>
                                ( "AND" [insertopt]=<usingOption> )* )?
                    ;

<usingOption> ::= "TIMESTAMP" <wholenumber>
                | "TTL" <wholenumber>
                ;
```

### UPDATE Statement
```bnf
<updateStatement> ::= "UPDATE" cf=<columnFamilyName>
                        ( "USING" [updateopt]=<usingOption>
                                  ( "AND" [updateopt]=<usingOption> )* )?
                        "SET" <assignment> ( "," <assignment> )*
                        "WHERE" <whereClause>
                        ( "IF" ( "EXISTS" | <conditions> ))?
                    ;

<assignment> ::= updatecol=<cident>
                    (( "=" update_rhs=( <term> | <cident> )
                                ( counterop=( "+" | "-" ) inc=<wholenumber>
                                | listadder="+" listcol=<cident> )? )
                    | ( indexbracket="[" <term> "]" "=" <term> )
                    | ( udt_field_dot="." udt_field=<identifier> "=" <term> ))
                ;

<conditions> ::=  <condition> ( "AND" <condition> )*
                ;

<condition> ::= conditioncol=<cident>
                    ( (( indexbracket="[" <term> "]" )
                      |( udt_field_dot="." udt_field=<identifier> )) )?
                    <condition_op_and_rhs>
              ;
```

### DELETE Statement
```bnf
<deleteStatement> ::= "DELETE" ( <deleteSelector> ( "," <deleteSelector> )* )?
                        "FROM" cf=<columnFamilyName>
                        ( "USING" [delopt]=<deleteOption> )?
                        "WHERE" <whereClause>
                        ( "IF" ( "EXISTS" | <conditions> ) )?
                    ;

<deleteSelector> ::= delcol=<cident>
                     ( ( "[" <term> "]" )
                     | ( "." <identifier> ) )?
                   ;

<deleteOption> ::= "TIMESTAMP" <wholenumber>
                 ;
```

### BATCH Statement
```bnf
<batchStatement> ::= "BEGIN" ( "UNLOGGED" | "COUNTER" )? "BATCH"
                        ( "USING" [batchopt]=<usingOption>
                                  ( "AND" [batchopt]=<usingOption> )* )?
                        [batchstmt]=<batchStatementMember> ";"?
                            ( [batchstmt]=<batchStatementMember> ";"? )*
                     "APPLY" "BATCH"
                   ;

<batchStatementMember> ::= <insertStatement>
                         | <updateStatement>
                         | <deleteStatement>
                         ;
```

### TRUNCATE Statement
```bnf
<truncateStatement> ::= "TRUNCATE" ("COLUMNFAMILY" | "TABLE")? cf=<columnFamilyName>
                      ;
```

---

## DDL: Keyspace Statements

### CREATE KEYSPACE
```bnf
<createKeyspaceStatement> ::= "CREATE" wat=( "KEYSPACE" | "SCHEMA" ) ("IF" "NOT" "EXISTS")?  ksname=<cfOrKsName>
                                "WITH" <property> ( "AND" <property> )*
                            ;
```

### ALTER KEYSPACE
```bnf
<alterKeyspaceStatement> ::= "ALTER" wat=( "KEYSPACE" | "SCHEMA" ) ("IF" "EXISTS")? ks=<alterableKeyspaceName>
                                 "WITH" <property> ( "AND" <property> )*
                           ;
```

### DROP KEYSPACE
```bnf
<dropKeyspaceStatement> ::= "DROP" "KEYSPACE" ("IF" "EXISTS")? ksname=<nonSystemKeyspaceName>
                          ;
```

---

## DDL: Table Statements

### CREATE TABLE
```bnf
<createColumnFamilyStatement> ::= "CREATE" wat=( "COLUMNFAMILY" | "TABLE" ) ("IF" "NOT" "EXISTS")?
                                    ( ks=<nonSystemKeyspaceName> dot="." )? cf=<cfOrKsName>
                                    "(" ( <singleKeyCfSpec> | <compositeKeyCfSpec> ) ")"
                                   ( "WITH" <cfamProperty> ( "AND" <cfamProperty> )* )?
                                ;

<cfamProperty> ::= <property>
                 | "COMPACT" "STORAGE" "CDC"
                 | "CLUSTERING" "ORDER" "BY" "(" <cfamOrdering>
                                                 ( "," <cfamOrdering> )* ")"
                 ;

<cfamOrdering> ::= [ordercol]=<cident> ( "ASC" | "DESC" )
                 ;

<singleKeyCfSpec> ::= [newcolname]=<cident> <storageType> "PRIMARY" "KEY"
                      ( "," [newcolname]=<cident> <storageType> )*
                    ;

<compositeKeyCfSpec> ::= [newcolname]=<cident> <storageType>
                         "," [newcolname]=<cident> <storageType> ( "static" )?
                         ( "," [newcolname]=<cident> <storageType> ( "static" )? )*
                         "," "PRIMARY" k="KEY" p="(" ( partkey=<pkDef> | [pkey]=<cident> )
                                                     ( c="," [pkey]=<cident> )* ")"
                       ;

<pkDef> ::= "(" [ptkey]=<cident> "," [ptkey]=<cident>
                               ( "," [ptkey]=<cident> )* ")"
          ;
```

### ALTER TABLE
```bnf
<alterTableStatement> ::= "ALTER" wat=( "COLUMNFAMILY" | "TABLE" ) ("IF" "EXISTS")? cf=<columnFamilyName>
                              <alterInstructions>
                       ;

<alterInstructions> ::= "ADD" ("IF" "NOT" "EXISTS")? newcol=<cident> <storageType> ("static")?
                      | "DROP" ("IF" "EXISTS")? existcol=<cident>
                      | "WITH" <cfamProperty> ( "AND" <cfamProperty> )*
                      | "RENAME" ("IF" "EXISTS")? existcol=<cident> "TO" newcol=<cident>
                         ( "AND" existcol=<cident> "TO" newcol=<cident> )*
                      | "ALTER" ("IF" "EXISTS")? existcol=<cident> ( <constraintsExpr> | <column_mask> | "DROP" ( "CHECK" | "MASKED" ) )
                      ;
```

### DROP TABLE
```bnf
<dropColumnFamilyStatement> ::= "DROP" ( "COLUMNFAMILY" | "TABLE" ) ("IF" "EXISTS")? cf=<columnFamilyName>
                              ;
```

---

## DDL: Index Statements

### CREATE INDEX
```bnf
<createIndexStatement> ::= "CREATE" "CUSTOM"? "INDEX" ("IF" "NOT" "EXISTS")? indexname=<idxName>? "ON"
                               cf=<columnFamilyName> "(" (
                                   col=<cident> |
                                   "keys(" col=<cident> ")" |
                                   "full(" col=<cident> ")"
                               ) ")"
                               ( "USING" <stringLiteral> ( "WITH" "OPTIONS" "=" <mapLiteral> )? )?
                         ;
```

### DROP INDEX
```bnf
<dropIndexStatement> ::= "DROP" "INDEX" ("IF" "EXISTS")? idx=<indexName>
                       ;

<indexName> ::= ( ksname=<idxOrKsName> dot="." )? idxname=<idxOrKsName> ;
```

---

## DDL: Materialized View Statements

### CREATE MATERIALIZED VIEW
```bnf
<createMaterializedViewStatement> ::= "CREATE" wat="MATERIALIZED" "VIEW" ("IF" "NOT" "EXISTS")? viewname=<materializedViewName>?
                                      "AS" "SELECT" <selectClause>
                                      "FROM" cf=<columnFamilyName>
                                      "WHERE" <cident> "IS" "NOT" "NULL" ( "AND" <cident> "IS" "NOT" "NULL")*
                                      "PRIMARY" "KEY" (<colList> | ( "(" <colList> ( "," <cident> )* ")" ))
                                      ( "WITH" <cfamProperty> ( "AND" <cfamProperty> )* )?
                                    ;
```

### DROP MATERIALIZED VIEW
```bnf
<dropMaterializedViewStatement> ::= "DROP" "MATERIALIZED" "VIEW" ("IF" "EXISTS")? mv=<materializedViewName>
                                  ;
```

---

## DDL: User-Defined Type Statements

### CREATE TYPE
```bnf
<createUserTypeStatement> ::= "CREATE" "TYPE" ("IF" "NOT" "EXISTS")? ( ks=<nonSystemKeyspaceName> dot="." )? typename=<cfOrKsName> "(" newcol=<cident> <storageType>
                                ( "," [newcolname]=<cident> <storageType> )*
                            ")"
                         ;
```

### ALTER TYPE
```bnf
<alterUserTypeStatement> ::= "ALTER" "TYPE" ("IF" "EXISTS")? ut=<userTypeName>
                               <alterTypeInstructions>
                             ;

<alterTypeInstructions> ::= "ADD" ("IF" "NOT" "EXISTS")? newcol=<cident> <storageType>
                           | "RENAME" ("IF" "EXISTS")? existcol=<cident> "TO" newcol=<cident>
                              ( "AND" existcol=<cident> "TO" newcol=<cident> )*
                           ;
```

### DROP TYPE
```bnf
<dropUserTypeStatement> ::= "DROP" "TYPE" ( "IF" "EXISTS" )? ut=<userTypeName>
                          ;
```

---

## DDL: Function & Aggregate Statements

### CREATE FUNCTION
```bnf
<createFunctionStatement> ::= "CREATE" ("OR" "REPLACE")? "FUNCTION"
                            ("IF" "NOT" "EXISTS")?
                            <userFunctionName>
                            ( "(" ( newcol=<cident> <storageType>
                              ( "," [newcolname]=<cident> <storageType> )* )?
                            ")" )?
                            ("RETURNS" "NULL" | "CALLED") "ON" "NULL" "INPUT"
                            "RETURNS" <storageType>
                            "LANGUAGE" <cident> "AS" <stringLiteral>
                         ;
```

### CREATE AGGREGATE
```bnf
<createAggregateStatement> ::= "CREATE" ("OR" "REPLACE")? "AGGREGATE"
                            ("IF" "NOT" "EXISTS")?
                            <userAggregateName>
                            ( "("
                                 ( <storageType> ( "," <storageType> )* )?
                              ")" )?
                            "SFUNC" <refUserFunctionName>
                            "STYPE" <storageType>
                            ( "FINALFUNC" <refUserFunctionName> )?
                            ( "INITCOND" <term> )?
                         ;
```

### DROP FUNCTION
```bnf
<dropFunctionStatement> ::= "DROP" "FUNCTION" ( "IF" "EXISTS" )? <userFunctionName>
                          ;
```

### DROP AGGREGATE
```bnf
<dropAggregateStatement> ::= "DROP" "AGGREGATE" ( "IF" "EXISTS" )? <userAggregateName>
                          ;
```

---

## DDL: Trigger Statements

### CREATE TRIGGER
```bnf
<createTriggerStatement> ::= "CREATE" "TRIGGER" ( "IF" "NOT" "EXISTS" )? <cident>
                               "ON" cf=<columnFamilyName> "USING" class=<stringLiteral>
                           ;
```

### DROP TRIGGER
```bnf
<dropTriggerStatement> ::= "DROP" "TRIGGER" ( "IF" "EXISTS" )? triggername=<cident>
                             "ON" cf=<columnFamilyName>
                         ;
```

---

## DCL: Authorization Statements

### GRANT
```bnf
<grantStatement> ::= "GRANT" <permissionExpr> "ON" <resource> "TO" <rolename>
                   ;

<permissionExpr> ::= ( [newpermission]=<permission> "PERMISSION"? ( "," [newpermission]=<permission> "PERMISSION"? )* )
                   | ( "ALL" "PERMISSIONS"? )
                   ;

<permission> ::= "AUTHORIZE"
               | "CREATE"
               | "ALTER"
               | "DROP"
               | "SELECT"
               | "MODIFY"
               | "DESCRIBE"
               | "EXECUTE"
               | "UNMASK"
               | "SELECT_MASKED"
               ;

<resource> ::= <dataResource>
             | <roleResource>
             | <functionResource>
             | <jmxResource>
             ;

<dataResource> ::= ( "ALL" "KEYSPACES" )
                 | ( "KEYSPACE" <keyspaceName> )
                 | ( "ALL" "TABLES" "IN" "KEYSPACE" <keyspaceName> )
                 | ( "TABLE"? <columnFamilyName> )
                 ;

<roleResource> ::= ("ALL" "ROLES")
                 | ("ROLE" <rolename>)
                 ;

<functionResource> ::= ( "ALL" "FUNCTIONS" ("IN KEYSPACE" <keyspaceName>)? )
                     | ( "FUNCTION" <functionAggregateName>
                           ( "(" ( newcol=<cident> <storageType>
                             ( "," [newcolname]=<cident> <storageType> )* )?
                           ")" )
                       )
                     ;

<jmxResource> ::= ( "ALL" "MBEANS")
                | ( ( "MBEAN" | "MBEANS" ) <stringLiteral> )
                ;
```

### REVOKE
```bnf
<revokeStatement> ::= "REVOKE" <permissionExpr> "ON" <resource> "FROM" <rolename>
                    ;
```

### LIST PERMISSIONS
```bnf
<listPermissionsStatement> ::= "LIST" <permissionExpr>
                                    ( "ON" <resource> )? ( "OF" <rolename> )? "NORECURSIVE"?
                             ;
```

---

## DCL: Role & User Management

### CREATE ROLE
```bnf
<createRoleStatement> ::= "CREATE" "ROLE" ("IF" "NOT" "EXISTS")? <rolename>
                              ( "WITH" <roleProperty> ("AND" <roleProperty>)*)?
                        ;

<roleProperty> ::= (("HASHED")? "PASSWORD") "=" <stringLiteral>
                 | "GENERATED" "PASSWORD"
                 | "OPTIONS" "=" <mapLiteral>
                 | "SUPERUSER" "=" <boolean>
                 | "LOGIN" "=" <boolean>
                 | "ACCESS" "TO" "DATACENTERS" <setLiteral>
                 | "ACCESS" "TO" "ALL" "DATACENTERS"
                 | "ACCESS" "FROM" "CIDRS" <setLiteral>
                 | "ACCESS" "FROM" "ALL" "CIDRS"
                 ;
```

### ALTER ROLE
```bnf
<alterRoleStatement> ::= "ALTER" "ROLE" ("IF" "EXISTS")? <rolename>
                              ( "WITH" <roleProperty> ("AND" <roleProperty>)*)
                       ;
```

### DROP ROLE
```bnf
<dropRoleStatement> ::= "DROP" "ROLE" ("IF" "EXISTS")? <rolename>
                      ;
```

### GRANT/REVOKE ROLE
```bnf
<grantRoleStatement> ::= "GRANT" <rolename> "TO" <rolename>
                       ;

<revokeRoleStatement> ::= "REVOKE" <rolename> "FROM" <rolename>
                        ;
```

### LIST ROLES
```bnf
<listRolesStatement> ::= "LIST" "ROLES"
                              ( "OF" <rolename> )? "NORECURSIVE"?
                       ;

<listSuperUsersStatement> ::= "LIST" "SUPERUSERS"
                       ;
```

### CREATE USER (Legacy)
```bnf
<createUserStatement> ::= "CREATE" "USER" ( "IF" "NOT" "EXISTS" )? <username>
                              ( ("WITH" ("HASHED")? "PASSWORD" <stringLiteral>) | ("WITH" "GENERATED" "PASSWORD") )?
                              ( "SUPERUSER" | "NOSUPERUSER" )?
                        ;

<username> ::= name=( <identifier> | <stringLiteral> )
             ;
```

### ALTER USER (Legacy)
```bnf
<alterUserStatement> ::= "ALTER" "USER" ("IF" "EXISTS")? <username>
                              ( ("WITH" "PASSWORD" <stringLiteral>) | ("WITH" "GENERATED" "PASSWORD") )?
                              ( "SUPERUSER" | "NOSUPERUSER" )?
                       ;
```

### DROP USER (Legacy)
```bnf
<dropUserStatement> ::= "DROP" "USER" ( "IF" "EXISTS" )? <username>
                      ;

<listUsersStatement> ::= "LIST" "USERS"
                       ;
```

---

## Additional Statements

### COMMENT
```bnf
<commentOnKeyspaceStatement> ::= "COMMENT" "ON" "KEYSPACE" ks=<keyspaceName> "IS" comment=( <stringLiteral> | "NULL" )
                               ;

<commentOnTableStatement> ::= "COMMENT" "ON" wat=( "COLUMNFAMILY" | "TABLE" ) cf=<columnFamilyName> "IS" comment=( <stringLiteral> | "NULL" )
                            ;

<commentOnColumnStatement> ::= "COMMENT" "ON" "COLUMN" cf=<columnFamilyName> dot="." col=<cident> "IS" comment=( <stringLiteral> | "NULL" )
                             ;

<commentOnTypeStatement> ::= "COMMENT" "ON" "TYPE" ut=<userTypeName> "IS" comment=( <stringLiteral> | "NULL" )
                           ;
```

### SECURITY LABEL
```bnf
<securityLabelOnKeyspaceStatement> ::= "SECURITY" "LABEL" "ON" "KEYSPACE" ks=<keyspaceName> "IS" label=( <stringLiteral> | "NULL" )
                                     ;

<securityLabelOnTableStatement> ::= "SECURITY" "LABEL" "ON" wat=( "COLUMNFAMILY" | "TABLE" ) cf=<columnFamilyName> "IS" label=( <stringLiteral> | "NULL" )
                                  ;

<securityLabelOnColumnStatement> ::= "SECURITY" "LABEL" "ON" "COLUMN" cf=<columnFamilyName> dot="." col=<cident> "IS" label=( <stringLiteral> | "NULL" )
                                   ;

<securityLabelOnTypeStatement> ::= "SECURITY" "LABEL" "ON" "TYPE" ut=<userTypeName> "IS" label=( <stringLiteral> | "NULL" )
                                 ;
```

### IDENTITY
```bnf
<addIdentityStatement> ::= "ADD" "IDENTITY" ("IF" "NOT" "EXISTS")? <stringLiteral> "TO" "ROLE" <rolename>
                         ;

<dropIdentityStatement> ::= "DROP" "IDENTITY" ("IF" "EXISTS")? <stringLiteral>
                          ;
```

---

## cqlsh Shell Commands

Source: `pylib/cqlshlib/cqlshhandling.py`

### DESCRIBE / DESC
```bnf
<describeCommand> ::= ( "DESCRIBE" | "DESC" )
                              ( ( "FUNCTIONS"
                                | "FUNCTION" udf=<anyFunctionName>
                                | "AGGREGATES"
                                | "AGGREGATE" uda=<userAggregateName>
                                | "KEYSPACES"
                                | "ONLY"? "KEYSPACE" ksname=<keyspaceName>?
                                | ( "COLUMNFAMILY" | "TABLE" ) cf=<columnFamilyName>
                                | "INDEX" idx=<indexName>
                                | "MATERIALIZED" "VIEW" mv=<materializedViewName>
                                | ( "COLUMNFAMILIES" | "TABLES" )
                                | "FULL"? "SCHEMA"
                                | "CLUSTER"
                                | "TYPES"
                                | "TYPE" ut=<userTypeName>
                                | (ksname=<keyspaceName> | cf=<columnFamilyName> | idx=<indexName> | mv=<materializedViewName>)
                                ) ("WITH" "INTERNALS")?
                              )
                  ;
```

### CONSISTENCY
```bnf
<consistencyCommand> ::= "CONSISTENCY" ( level=<consistencyLevel> )?
                       ;

<consistencyLevel> ::= "ANY"
                     | "ONE"
                     | "TWO"
                     | "THREE"
                     | "QUORUM"
                     | "ALL"
                     | "LOCAL_QUORUM"
                     | "EACH_QUORUM"
                     | "SERIAL"
                     | "LOCAL_SERIAL"
                     | "LOCAL_ONE"
                     | "NODE_LOCAL"
                     ;
```

### SERIAL CONSISTENCY
```bnf
<serialConsistencyCommand> ::= "SERIAL" "CONSISTENCY" ( level=<serialConsistencyLevel> )?
                             ;

<serialConsistencyLevel> ::= "SERIAL"
                           | "LOCAL_SERIAL"
                           ;
```

### SHOW
```bnf
<showCommand> ::= "SHOW" what=( "VERSION" | "HOST" | "SESSION" sessionid=<uuid> | "REPLICAS" token=<integer> (keyspace=<keyspaceName>)? )
                ;
```

### SOURCE
```bnf
<sourceCommand> ::= "SOURCE" fname=<stringLiteral>
                  ;
```

### CAPTURE
```bnf
<captureCommand> ::= "CAPTURE" ( switch=( <stringLiteral> | "OFF" ) )?
                   ;
```

### COPY
```bnf
<copyCommand> ::= "COPY" cf=<columnFamilyName>
                         ( "(" [colnames]=<colname> ( "," [colnames]=<colname> )* ")" )?
                         ( dir="FROM" ( fname=<stringLiteral> | "STDIN" )
                         | dir="TO"   ( fname=<stringLiteral> | "STDOUT" ) )
                         ( "WITH" <copyOption> ( "AND" <copyOption> )* )?
                ;

<copyOption> ::= [optnames]=(<identifier>|<reserved_identifier>) "=" [optvals]=<copyOptionVal>
               ;

<copyOptionVal> ::= <identifier>
                  | <reserved_identifier>
                  | <term>
                  ;
```

### TRACING
```bnf
<tracingCommand> ::= "TRACING" ( switch=( "ON" | "OFF" ) )?
                   ;
```

### EXPAND
```bnf
<expandCommand> ::= "EXPAND" ( switch=( "ON" | "OFF" ) )?
                   ;
```

### PAGING
```bnf
<pagingCommand> ::= "PAGING" ( switch=( "ON" | "OFF" | <wholenumber>) )?
                  ;
```

### ELAPSED
```bnf
<elapsedCommand> ::= "ELAPSED" ( switch=( "ON" | "OFF" ) )?
                  ;
```

### LOGIN
```bnf
<loginCommand> ::= "LOGIN" username=<username> (password=<stringLiteral>)?
                 ;
```

### DEBUG
```bnf
<debugCommand> ::= "DEBUG" "THINGS"?
                 ;
```

### HELP
```bnf
<helpCommand> ::= ( "HELP" | "?" ) [topic]=( /[a-z_]*/ )*
                ;
```

### EXIT/QUIT
```bnf
<exitCommand> ::= "exit" | "quit"
                ;
```

### CLEAR/CLS
```bnf
<clearCommand> ::= "CLEAR" | "CLS"
                 ;
```

### HISTORY
```bnf
<historyCommand> ::= "history" (n=<wholenumber>)?
                    ;
```

---

## Terminal Lexical Rules

```bnf
<endline> ::= /\n/ ;

JUNK ::= /([ \t\r\f\v]+|(--|[/][/])[^\n\r]*([\n\r]|$)|[/][*].*?[*][/])/ ;

<stringLiteral> ::= <quotedStringLiteral>
                  | <pgStringLiteral> ;
<quotedStringLiteral> ::= /'([^']|'')*'/ ;
<pgStringLiteral> ::= /\$\$(?:(?!\$\$).)*\$\$/;
<quotedName> ::=    /"([^"]|"")*"/ ;

<unclosedPgString>::= /\$\$(?:(?!\$\$).)*/ ;
<unclosedString>  ::= /'([^']|'')*/ ;
<unclosedName>    ::= /"([^"]|"")*/ ;
<unclosedComment> ::= /[/][*].*$/ ;

<float> ::=         /-?[0-9]+\.[0-9]+/ ;
<uuid> ::=          /[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}/ ;
<blobLiteral> ::=    /0x[0-9a-f]+/ ;
<wholenumber> ::=   /[0-9]+/ ;
<identifier> ::=    /[a-z][a-z0-9_]*/ ;
<colon> ::=         ":" ;
<star> ::=          "*" ;
<endtoken> ::=      ";" ;
<op> ::=            /[-+=%/,().]/ ;
<cmp> ::=           /[<>!]=?/ ;
<brackets> ::=      /[][{}]/ ;

<integer> ::= "-"? <wholenumber> ;
<boolean> ::= "true"
            | "false"
            ;
```

---

## GitHub Permalinks

All rules above are taken from:

1. **CQL Statement Rules**: https://github.com/apache/cassandra/blob/daadd25fb29ae76110e0764ce568272c7fec5682/pylib/cqlshlib/cql3handling.py#L168-L1821

2. **cqlsh Shell Commands**: https://github.com/apache/cassandra/blob/daadd25fb29ae76110e0764ce568272c7fec5682/pylib/cqlshlib/cqlshhandling.py#L56-L251

---

## Key Design Notes for Translation to Rust

1. **Two-part grammar**: CQL statements are defined in `cql3handling.py`, shell commands in `cqlshhandling.py`

2. **Naming conventions**: 
   - Rules are wrapped in `< >` angle brackets (non-terminals)
   - Terminals are quoted strings like `"SELECT"` or regex patterns
   - Capture groups use `name=` prefix for bindings (e.g., `cf=<columnFamilyName>`)

3. **Optional/repetition operators**:
   - `( ... )?` — optional
   - `( ... )*` — zero or more
   - `[name]=<rule>` — repeating capture group (stored as array)

4. **Authorization model**: Permissions, resources, and roles follow a comprehensive hierarchy for GRANT/REVOKE

5. **Shell commands end with newline** — Tracked in `commands_end_with_newline` set (see `cqlshhandling.py` line 22-43)

