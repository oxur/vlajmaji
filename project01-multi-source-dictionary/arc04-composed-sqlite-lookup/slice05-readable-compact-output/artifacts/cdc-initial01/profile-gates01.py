"""Prospective same-ID S05 warm-gate evaluator; no product execution.

The observer adapter and exact measured timer applicability still require frozen review.
"""
import math
import statistics

IMPLEMENTATIONS = ("candidate-full", "compact", "readable")

def milliseconds(value):
    assert type(value) in (int, float) and math.isfinite(value) and value >= 0
    return value

def metric(values, median_ceiling, maximum_ceiling):
    assert len(values) == 3
    values = [milliseconds(value) for value in values]
    median, maximum = statistics.median(values), max(values)
    return {"observations": 3, "median_ms": median, "max_ms": maximum,
            "median_ceiling_ms": median_ceiling, "max_ceiling_ms": maximum_ceiling,
            "passed": median <= median_ceiling and maximum <= maximum_ceiling}

def evaluate(requests, observations):
    measured = {row["id"]: row for row in requests if not row.get("prime", False)}
    assert len(measured) == 27
    assert len(observations) == 3 * 3 * 27
    groups = {}
    for row in observations:
        assert row["implementation"] in IMPLEMENTATIONS
        assert type(row["round"]) is int and row["round"] in (1, 2, 3)
        assert row["prime"] is False and row["id"] in measured
        expected = measured[row["id"]]
        assert row["operation"] == expected["operation"]
        error = expected.get("expected_error")
        assert row["ok"] is (error is None)
        assert row["error_category"] == error
        key = (row["implementation"], row["id"])
        rounds = groups.setdefault(key, {})
        assert row["round"] not in rounds
        rounds[row["round"]] = row
    results = []
    for implementation in IMPLEMENTATIONS:
        for identity, request in measured.items():
            rounds = groups[(implementation, identity)]
            assert set(rounds) == {1, 2, 3}
            rows = [rounds[index] for index in (1, 2, 3)]
            # Whole resolution construction is charged; projection is never subtracted.
            field = "query_total_ms" if request["operation"] == "reference" else "lookup_ms"
            warm = metric([row[field] for row in rows], 250, 750)
            projection = None
            if request.get("expected_error"):
                assert all(row["projection_ms"] is None for row in rows)
            elif implementation in ("compact", "readable") or identity == "farvi-all":
                values = [milliseconds(row["projection_ms"]) + milliseconds(row["serialization_ms"]) for row in rows]
                limits = (500, 1000) if implementation == "candidate-full" else (50, 150)
                projection = metric(values, *limits)
            results.append({"implementation": implementation, "id": identity,
                            "warm": warm, "projection_and_serialization": projection})
    return {"ok": all(row["warm"]["passed"] and
                       (row["projection_and_serialization"] is None or row["projection_and_serialization"]["passed"])
                       for row in results), "groups": results}
