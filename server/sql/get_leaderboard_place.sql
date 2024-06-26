SELECT
    rownum as place
FROM
    (
        SELECT
            user_id,
            RANK() OVER (
                ORDER BY
                    total_rating DESC
            ) as rownum
        FROM
            user_period_data
        WHERE
            period_type = $1
    ) as leaderboard
WHERE
    user_id = $2
