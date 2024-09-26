-- ------------------------------
-- OPTION
-- ------------------------------

OPTION IMPORT;

-- ------------------------------
-- ACCESSES
-- ------------------------------

DEFINE ACCESS user ON DATABASE TYPE RECORD SIGNUP (CREATE user CONTENT { email: $email, name: $name, password: crypto::argon2::generate($password) }) SIGNIN (SELECT * FROM user WHERE email = $email AND crypto::argon2::compare(password, $password)) WITH JWT ALGORITHM HS512 KEY 'M7y2CZGZELUlked6VIbafJU37Pc1V3csA5qwqarxJYrY1zZoa4J0hprTE7Eu5qBIO2csHuDCCcWfOihBh41snNAfEVtc4HPoTOiR9kMe9onLGKXKOt9LQVSulXFDKMZ4' WITH ISSUER KEY 'M7y2CZGZELUlked6VIbafJU37Pc1V3csA5qwqarxJYrY1zZoa4J0hprTE7Eu5qBIO2csHuDCCcWfOihBh41snNAfEVtc4HPoTOiR9kMe9onLGKXKOt9LQVSulXFDKMZ4' DURATION FOR TOKEN 15m, FOR SESSION 12h;

-- ------------------------------
-- FUNCTIONS
-- ------------------------------

DEFINE FUNCTION fn::exists($pk: any) { RETURN count((SELECT * FROM type::thing($pk))) > 0; } PERMISSIONS FULL;

-- ------------------------------
-- TABLE: card
-- ------------------------------

DEFINE TABLE card TYPE NORMAL SCHEMAFULL PERMISSIONS FOR select, create, update, delete WHERE user = $auth.id;

DEFINE FIELD back ON card TYPE option<string> PERMISSIONS FULL;
DEFINE FIELD data ON card FLEXIBLE TYPE option<object> PERMISSIONS FULL;
DEFINE FIELD importance ON card TYPE int DEFAULT 0 ASSERT $value >= 0 AND $value <= 10 PERMISSIONS FULL;
DEFINE FIELD difficulty ON card TYPE int DEFAULT 0 ASSERT $value >= 0 AND $value <= 10 PERMISSIONS FULL;
DEFINE FIELD front ON card TYPE option<string> PERMISSIONS FULL;
DEFINE FIELD hints ON card TYPE array<string> DEFAULT [] PERMISSIONS FULL;
DEFINE FIELD tags ON card TYPE set<record<tag>> DEFAULT [] PERMISSIONS FULL;
DEFINE FIELD time ON card TYPE object DEFAULT {  } PERMISSIONS FULL;
DEFINE FIELD time.created_at ON card TYPE datetime DEFAULT time::now() VALUE $before OR time::now() PERMISSIONS FULL;
DEFINE FIELD time.updated_at ON card TYPE datetime DEFAULT time::now() VALUE time::now() PERMISSIONS FULL;
DEFINE FIELD title ON card TYPE string PERMISSIONS FULL;
DEFINE FIELD user ON card TYPE record<user> ASSERT $value != NONE AND $value != NULL AND fn::exists(<string> $value) PERMISSIONS FULL;

-- ------------------------------
-- TABLE: card_group
-- ------------------------------

DEFINE TABLE card_group TYPE NORMAL SCHEMAFULL PERMISSIONS FOR select, create, update, delete WHERE user = $auth.id;
DEFINE FIELD data ON card_group FLEXIBLE TYPE option<object> PERMISSIONS FULL;
DEFINE FIELD importance ON card_group TYPE int DEFAULT 0 ASSERT $value >= 0 AND $value <= 10 PERMISSIONS FULL;
DEFINE FIELD difficulty ON card_group TYPE int DEFAULT 0 ASSERT $value >= 0 AND $value <= 10 PERMISSIONS FULL;
DEFINE FIELD title ON card_group TYPE string PERMISSIONS FULL;
DEFINE FIELD time ON card_group TYPE object DEFAULT {  } PERMISSIONS FULL;
DEFINE FIELD time.created_at ON card_group TYPE datetime DEFAULT time::now() VALUE $before OR time::now() PERMISSIONS FULL;
DEFINE FIELD time.updated_at ON card_group TYPE datetime DEFAULT time::now() VALUE time::now() PERMISSIONS FULL;
DEFINE FIELD cards ON card_group TYPE array<record<card>> DEFAULT [] PERMISSIONS FULL;
DEFINE FIELD user ON card_group TYPE record<user> ASSERT $value != NONE AND $value != NULL AND fn::exists(<string> $value) PERMISSIONS FULL;
DEFINE FIELD tags ON card_group TYPE set<record<tag>> DEFAULT [] PERMISSIONS FULL;

-- ------------------------------
-- TABLE: deck
-- ------------------------------

DEFINE TABLE deck TYPE NORMAL SCHEMAFULL PERMISSIONS FOR select, create, update, delete WHERE user = $auth.id;

DEFINE FIELD description ON deck TYPE option<string> PERMISSIONS FULL;
DEFINE FIELD parent ON deck TYPE option<record<deck>> ASSERT $value == NONE OR $value == NULL OR fn::exists(<string> $value) PERMISSIONS FULL;
DEFINE FIELD settings ON deck FLEXIBLE TYPE option<object> DEFAULT {  } PERMISSIONS FULL;
DEFINE FIELD settings.daily_limit ON deck TYPE int DEFAULT 0 PERMISSIONS FULL;
DEFINE FIELD tags ON deck TYPE set<record<tag>> DEFAULT [] PERMISSIONS FULL;
DEFINE FIELD time ON deck TYPE object DEFAULT {  } PERMISSIONS FULL;
DEFINE FIELD time.created_at ON deck TYPE datetime DEFAULT time::now() VALUE $before OR time::now() PERMISSIONS FULL;
DEFINE FIELD time.updated_at ON deck TYPE datetime DEFAULT time::now() VALUE time::now() PERMISSIONS FULL;
DEFINE FIELD title ON deck TYPE string ASSERT $value != NONE AND $value != NULL PERMISSIONS FULL;
DEFINE FIELD user ON deck TYPE record<user> ASSERT $value != NONE AND $value != NULL AND fn::exists(<string> $value) PERMISSIONS FULL;

-- ------------------------------
-- TABLE: deck_card
-- ------------------------------

DEFINE TABLE deck_card TYPE RELATION IN deck OUT card SCHEMAFULL PERMISSIONS FOR select, create, update, delete WHERE in.user == $auth.id AND out.user == $auth.id;

DEFINE FIELD time ON deck_card TYPE object DEFAULT {  } PERMISSIONS FULL;
DEFINE FIELD time.created_at ON deck_card TYPE datetime DEFAULT time::now() VALUE $before OR time::now() PERMISSIONS FULL;
DEFINE FIELD time.updated_at ON deck_card TYPE datetime DEFAULT time::now() VALUE time::now() PERMISSIONS FULL;

DEFINE INDEX unique_in_out ON deck_card FIELDS in, out UNIQUE;

-- ------------------------------
-- TABLE: deck_card_group
-- ------------------------------

DEFINE TABLE deck_card_group TYPE RELATION IN deck OUT card_group SCHEMAFULL PERMISSIONS FOR select, create, update, delete WHERE in.user == $auth.id AND out.user == $auth.id;

DEFINE FIELD time ON deck_card_group TYPE object DEFAULT {  } PERMISSIONS FULL;
DEFINE FIELD time.created_at ON deck_card_group TYPE datetime DEFAULT time::now() VALUE $before OR time::now() PERMISSIONS FULL;
DEFINE FIELD time.updated_at ON deck_card_group TYPE datetime DEFAULT time::now() VALUE time::now() PERMISSIONS FULL;

DEFINE INDEX unique_in_out ON deck_card_group FIELDS in, out UNIQUE;

-- ------------------------------
-- TABLE: global_settings
-- ------------------------------

DEFINE TABLE global_settings TYPE ANY SCHEMAFULL PERMISSIONS FOR select, create, update, delete WHERE user = $auth.id;

DEFINE FIELD daily_limit ON global_settings TYPE int DEFAULT 0 ASSERT $value >= 0 AND $value <= 10000 PERMISSIONS FULL;
DEFINE FIELD time ON global_settings TYPE object DEFAULT {  } PERMISSIONS FULL;
DEFINE FIELD time.created_at ON global_settings TYPE datetime DEFAULT time::now() VALUE $before OR time::now() PERMISSIONS FULL;
DEFINE FIELD time.updated_at ON global_settings TYPE datetime DEFAULT time::now() VALUE time::now() PERMISSIONS FULL;
DEFINE FIELD timetable ON global_settings TYPE array<array<duration, 2>> DEFAULT [] PERMISSIONS FULL;
DEFINE FIELD user ON global_settings TYPE record<user> ASSERT $value != NONE AND $value != NULL AND fn::exists(<string> $value) PERMISSIONS FULL;

DEFINE INDEX unique_for_user ON global_settings FIELDS user UNIQUE;

-- ------------------------------
-- TABLE: history
-- ------------------------------

DEFINE TABLE history TYPE ANY SCHEMAFULL PERMISSIONS FOR select, create, update, delete WHERE user = $auth.id;
DEFINE FIELD user ON history TYPE record<user> ASSERT $value != NONE AND $value != NULL AND fn::exists(<string> $value) PERMISSIONS FULL;
DEFINE FIELD deck_card ON history TYPE option<record<deck_card>> PERMISSIONS FULL;
DEFINE FIELD deck_card_group ON history TYPE option<record<deck_card_group>> PERMISSIONS FULL;
DEFINE FIELD difficulty ON history TYPE int DEFAULT 0 ASSERT $value >= 0 AND $value <= 10 PERMISSIONS FULL;
DEFINE FIELD time ON history TYPE option<object> DEFAULT {  } PERMISSIONS FULL;
DEFINE FIELD time.created_at ON history TYPE datetime DEFAULT time::now() VALUE $before OR time::now() PERMISSIONS FULL;
DEFINE FIELD time.updated_at ON history TYPE datetime DEFAULT time::now() VALUE time::now() PERMISSIONS FULL;

-- ------------------------------
-- TABLE: schedule
-- ------------------------------

DEFINE TABLE schedule TYPE ANY SCHEMALESS PERMISSIONS FOR select, create, update, delete WHERE user = $auth.id;

DEFINE FIELD deck_card ON schedule TYPE record<deck_card> PERMISSIONS FULL;
DEFINE FIELD time ON schedule TYPE option<object> DEFAULT {  } PERMISSIONS FULL;
DEFINE FIELD time.created_at ON schedule TYPE datetime DEFAULT time::now() VALUE $before OR time::now() PERMISSIONS FULL;
DEFINE FIELD time.updated_at ON schedule TYPE datetime DEFAULT time::now() VALUE time::now() PERMISSIONS FULL;
DEFINE FIELD time.fire_at ON schedule TYPE datetime PERMISSIONS FULL;
DEFINE FIELD user ON schedule TYPE record<user> ASSERT $value != NONE AND $value != NULL AND fn::exists(<string> $value) PERMISSIONS FULL;

-- ------------------------------
-- TABLE: tag
-- ------------------------------

DEFINE TABLE tag TYPE NORMAL SCHEMAFULL PERMISSIONS FOR select, create, update, delete WHERE user = $auth.id;

DEFINE FIELD name ON tag TYPE string ASSERT $value != NONE AND $value != NULL PERMISSIONS FULL;
DEFINE FIELD slug ON tag TYPE string ASSERT $value != NONE AND $value != NULL PERMISSIONS FULL;
DEFINE FIELD user ON tag TYPE record<user> ASSERT $value != NONE AND $value != NULL AND fn::exists(<string> $value) PERMISSIONS FULL;

DEFINE FIELD time ON tag TYPE object DEFAULT {  } PERMISSIONS FULL;
DEFINE FIELD time.created_at ON tag TYPE datetime DEFAULT time::now() VALUE $before OR time::now() PERMISSIONS FULL;
DEFINE FIELD time.updated_at ON tag TYPE datetime DEFAULT time::now() VALUE time::now() PERMISSIONS FULL;

DEFINE INDEX unique_user_slug ON tag FIELDS user, slug UNIQUE;

-- ------------------------------
-- TABLE: user
-- ------------------------------

DEFINE TABLE user TYPE ANY SCHEMAFULL PERMISSIONS FOR select, update, delete WHERE id = $auth.id, FOR create NONE;

DEFINE FIELD email ON user TYPE string ASSERT string::is::email($value) PERMISSIONS FULL;
DEFINE FIELD name ON user TYPE string PERMISSIONS FULL;
DEFINE FIELD password ON user TYPE string PERMISSIONS FULL;
DEFINE FIELD time ON user TYPE object DEFAULT {  } PERMISSIONS FULL;
DEFINE FIELD time.created_at ON user TYPE datetime DEFAULT time::now() VALUE $before OR time::now() PERMISSIONS FULL;
DEFINE FIELD time.updated_at ON user TYPE datetime DEFAULT time::now() VALUE time::now() PERMISSIONS FULL;

DEFINE INDEX email ON user FIELDS email UNIQUE;

-- ------------------------------
-- TABLE: binding
-- ------------------------------

DEFINE TABLE binding TYPE ANY SCHEMAFULL PERMISSIONS FOR select, update, delete WHERE id = $auth.id, FOR create NONE;

DEFINE FIELD user ON binding TYPE record<user> ASSERT $value != NONE AND $value != NULL AND fn::exists(<string> $value) PERMISSIONS FULL;
DEFINE FIELD source_id ON binding TYPE string PERMISSIONS FULL;
DEFINE FIELD type_name ON binding TYPE string PERMISSIONS FULL;
DEFINE FIELD data ON binding FLEXIBLE TYPE option<object> PERMISSIONS FULL;

DEFINE FIELD time ON binding TYPE object DEFAULT {  } PERMISSIONS FULL;
DEFINE FIELD time.created_at ON binding TYPE datetime DEFAULT time::now() VALUE $before OR time::now() PERMISSIONS FULL;
DEFINE FIELD time.updated_at ON binding TYPE datetime DEFAULT time::now() VALUE time::now() PERMISSIONS FULL;

