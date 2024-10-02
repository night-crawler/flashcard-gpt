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
DEFINE FIELD time.created_at ON card TYPE datetime DEFAULT time::now() VALUE $value OR $before OR time::now() PERMISSIONS FULL;
DEFINE FIELD time.updated_at ON card TYPE datetime DEFAULT time::now() VALUE $value OR time::now() PERMISSIONS FULL;
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
DEFINE FIELD time.created_at ON card_group TYPE datetime DEFAULT time::now() VALUE $value OR $before OR time::now() PERMISSIONS FULL;
DEFINE FIELD time.updated_at ON card_group TYPE datetime DEFAULT time::now() VALUE $value OR time::now() PERMISSIONS FULL;
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
DEFINE FIELD time.created_at ON deck TYPE datetime DEFAULT time::now() VALUE $value OR $before OR time::now() PERMISSIONS FULL;
DEFINE FIELD time.updated_at ON deck TYPE datetime DEFAULT time::now() VALUE $value OR time::now() PERMISSIONS FULL;
DEFINE FIELD title ON deck TYPE string ASSERT $value != NONE AND $value != NULL PERMISSIONS FULL;
DEFINE FIELD user ON deck TYPE record<user> ASSERT $value != NONE AND $value != NULL AND fn::exists(<string> $value) PERMISSIONS FULL;

-- ------------------------------
-- TABLE: deck_card
-- ------------------------------

DEFINE TABLE deck_card TYPE RELATION IN deck OUT card SCHEMAFULL PERMISSIONS FOR select, create, update, delete WHERE in.user == $auth.id AND out.user == $auth.id;

DEFINE FIELD time ON deck_card TYPE object DEFAULT {  } PERMISSIONS FULL;
DEFINE FIELD time.created_at ON deck_card TYPE datetime DEFAULT time::now() VALUE $value OR $before OR time::now() PERMISSIONS FULL;
DEFINE FIELD time.updated_at ON deck_card TYPE datetime DEFAULT time::now() VALUE $value OR time::now() PERMISSIONS FULL;

DEFINE INDEX unique_in_out ON deck_card FIELDS in, out UNIQUE;

-- ------------------------------
-- TABLE: deck_card_group
-- ------------------------------

DEFINE TABLE deck_card_group TYPE RELATION IN deck OUT card_group SCHEMAFULL PERMISSIONS FOR select, create, update, delete WHERE in.user == $auth.id AND out.user == $auth.id;

DEFINE FIELD time ON deck_card_group TYPE object DEFAULT {  } PERMISSIONS FULL;
DEFINE FIELD time.created_at ON deck_card_group TYPE datetime DEFAULT time::now() VALUE $value OR $before OR time::now() PERMISSIONS FULL;
DEFINE FIELD time.updated_at ON deck_card_group TYPE datetime DEFAULT time::now() VALUE $value OR time::now() PERMISSIONS FULL;

DEFINE INDEX unique_in_out ON deck_card_group FIELDS in, out UNIQUE;

-- ------------------------------
-- TABLE: global_settings
-- ------------------------------

DEFINE TABLE global_settings TYPE ANY SCHEMAFULL PERMISSIONS FOR select, create, update, delete WHERE user = $auth.id;

DEFINE FIELD daily_limit ON global_settings TYPE int DEFAULT 0 ASSERT $value >= 0 AND $value <= 10000 PERMISSIONS FULL;
DEFINE FIELD time ON global_settings TYPE object DEFAULT {  } PERMISSIONS FULL;
DEFINE FIELD time.created_at ON global_settings TYPE datetime DEFAULT time::now() VALUE $value OR $before OR time::now() PERMISSIONS FULL;
DEFINE FIELD time.updated_at ON global_settings TYPE datetime DEFAULT time::now() VALUE $value OR time::now() PERMISSIONS FULL;
DEFINE FIELD timetable ON global_settings TYPE array<array<duration, 2>> DEFAULT [] PERMISSIONS FULL;
DEFINE FIELD timezone ON global_settings TYPE string DEFAULT "Europe/Dublin" PERMISSIONS FULL;
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
DEFINE FIELD time.created_at ON history TYPE datetime DEFAULT time::now() VALUE $value OR $before OR time::now() PERMISSIONS FULL;
DEFINE FIELD time.updated_at ON history TYPE datetime DEFAULT time::now() VALUE $value OR time::now() PERMISSIONS FULL;

DEFINE INDEX created_at_index ON TABLE history COLUMNS time.created_at;
DEFINE INDEX deck_card_index ON TABLE history COLUMNS deck_card;
DEFINE INDEX deck_card_group_index ON TABLE history COLUMNS deck_card_group;

-- ------------------------------
-- TABLE: schedule
-- ------------------------------

DEFINE TABLE schedule TYPE ANY SCHEMALESS PERMISSIONS FOR select, create, update, delete WHERE user = $auth.id;

DEFINE FIELD deck_card ON schedule TYPE record<deck_card> PERMISSIONS FULL;
DEFINE FIELD time ON schedule TYPE option<object> DEFAULT {  } PERMISSIONS FULL;
DEFINE FIELD time.created_at ON schedule TYPE datetime DEFAULT time::now() VALUE $value OR $before OR time::now() PERMISSIONS FULL;
DEFINE FIELD time.updated_at ON schedule TYPE datetime DEFAULT time::now() VALUE $value OR time::now() PERMISSIONS FULL;
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
DEFINE FIELD time.created_at ON tag TYPE datetime DEFAULT time::now() VALUE $value OR $before OR time::now() PERMISSIONS FULL;
DEFINE FIELD time.updated_at ON tag TYPE datetime DEFAULT time::now() VALUE $value OR time::now() PERMISSIONS FULL;

DEFINE INDEX unique_user_slug ON tag FIELDS user, slug UNIQUE;

-- ------------------------------
-- TABLE: user
-- ------------------------------

DEFINE TABLE user TYPE ANY SCHEMAFULL PERMISSIONS FOR select, update, delete WHERE id = $auth.id, FOR create NONE;

DEFINE FIELD email ON user TYPE string ASSERT string::is::email($value) PERMISSIONS FULL;
DEFINE FIELD name ON user TYPE string PERMISSIONS FULL;
DEFINE FIELD password ON user TYPE string PERMISSIONS FULL;
DEFINE FIELD time ON user TYPE object DEFAULT {  } PERMISSIONS FULL;
DEFINE FIELD time.created_at ON user TYPE datetime DEFAULT time::now() VALUE $value OR $before OR time::now() PERMISSIONS FULL;
DEFINE FIELD time.updated_at ON user TYPE datetime DEFAULT time::now() VALUE $value OR time::now() PERMISSIONS FULL;

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
DEFINE FIELD time.created_at ON binding TYPE datetime DEFAULT time::now() VALUE $value OR $before OR time::now() PERMISSIONS FULL;
DEFINE FIELD time.updated_at ON binding TYPE datetime DEFAULT time::now() VALUE $value OR time::now() PERMISSIONS FULL;

-- functions

DEFINE FUNCTION OVERWRITE fn::trend($card_group: record) {

    return function($card_group) {
        const [card_group] = arguments;

        const dataQuery = new surrealdb.Query(`
            SELECT difficulty, time::unix(time.created_at) AS ts
            FROM history
            WHERE
                  (
                      deck_card_group.out = ${card_group} or
                      deck_card.out = ${card_group}
                  ) AND
                difficulty != NONE
            ORDER BY ts DESC
            LIMIT 10
            PARALLEL
        `);

        const data = await surrealdb.query(dataQuery);
        data.reverse();

        const times = data.map(pair => pair.ts);
        const difficulties = data.map(pair => pair.difficulty);

        const mean_time = times.reduce((sum, t) => sum + t, 0) / times.length;
        const mean_difficulty = difficulties.reduce((sum, d) => sum + d, 0) / difficulties.length;

        const time_diff = times.map(t => t - mean_time);
        const difficulty_diff = difficulties.map(d => d - mean_difficulty);

        const numerator = time_diff.reduce((sum, t, i) => sum + (t * difficulty_diff[i]), 0);
        const denominator = time_diff.reduce((sum, t) => sum + (t * t), 0);

        const result = {
            slope: 10,
            mean_difficulty: mean_difficulty,
        };

        if (denominator != 0) {
            result.slope = 1 + numerator / denominator ;
        }

        return result;
    };
} PERMISSIONS FULL;



DEFINE FUNCTION OVERWRITE fn::since_last($pk: record) {
    let $last = (
        select time.created_at as ts
        from history
        where deck_card_group.out = $pk or deck_card.out = $pk
        order by ts desc
        limit 1
    )[0].ts;
    return time::now() - ($last or (time::now() - 30d));
};


DEFINE FUNCTION OVERWRITE fn::rank($importance: int, $difficulty: int, $slope: float, $last_seen: duration) {
    return ($importance + 1) * ($difficulty + 1) * $slope * $last_seen.mins();
};


DEFINE FUNCTION OVERWRITE fn::deck_card_group_answered_times($pk: record, $since: datetime) {
    return (select 
        deck_card_group, count() from history 
        where 
            deck_card_group = $pk and
            time.created_at >= $since
        group by deck_card_group
    )[0].count or 0;
};


DEFINE FUNCTION OVERWRITE fn::deck_card_answered_times($pk: record, $since: datetime) {
    return (select 
        deck_card, count() from history 
        where 
            deck_card = $pk and
            time.created_at >= $since
        group by deck_card
    )[0].count or 0;
};

DEFINE FUNCTION OVERWRITE fn::appears_in_card_groups_in_this_deck($pk: record, $deck: record) {
    return (select count(), in, out
        from deck_card_group
        where
            out.cards contains $pk and
            in = $deck
        group by in, out
        limit 1
    )[0].count or 0;
};

DEFINE FUNCTION OVERWRITE fn::num_answers_for_deck($pk: record, $since: datetime) {
    return count(
        select
            count(),
            deck_card_group.in ?: deck_card.in as deck,
            deck_card ?: deck_card_group as what
        from history
            where
                time.created_at >= $since and (
                    deck_card_group.in = $pk or
                    deck_card.in = $pk
                )
        group by deck, what
    )
};
