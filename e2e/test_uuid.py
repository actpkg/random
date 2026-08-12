import re

import pytest

UUID_V4 = re.compile(r"^[0-9a-f]{8}-[0-9a-f]{4}-4[0-9a-f]{3}-[89ab][0-9a-f]{3}-[0-9a-f]{12}$")
UUID_V7 = re.compile(r"^[0-9a-f]{8}-[0-9a-f]{4}-7[0-9a-f]{3}-[89ab][0-9a-f]{3}-[0-9a-f]{12}$")

CASES = [
    (None, UUID_V4),  # v4 (default)
    (7, UUID_V7),
]


@pytest.mark.parametrize("version,pattern", CASES)
async def test_uuid_matches_version_shape(client, version, pattern):
    args = {} if version is None else {"version": version}
    result = await client.call_tool("uuid", args)
    assert re.fullmatch(pattern, result.content[0].text)


async def test_rejects_an_unsupported_version(client, expect_error):
    await expect_error(client, "uuid", {"version": 3}, "std:invalid-args")
