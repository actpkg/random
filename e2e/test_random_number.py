import re

import pytest

CASES = [
    # (args, expected-regex)
    ({}, r"^-?\d+$"),  # default range 0-100
    ({"min": 1, "max": 6}, r"^[1-6]$"),
]


@pytest.mark.parametrize("args,pattern", CASES)
async def test_random_number_matches_range_shape(client, args, pattern):
    result = await client.call_tool("random_number", args)
    assert re.fullmatch(pattern, result.content[0].text)


async def test_rejects_an_invalid_range(client, expect_error):
    await expect_error(client, "random_number", {"min": 10, "max": 1}, "std:invalid-args")
