import re

import pytest

CASES = [
    # (args, expected-regex)
    ({"length": 16}, r"^[a-zA-Z0-9]{16}$"),  # default alphanumeric
    ({"length": 8, "charset": "hex"}, r"^[0-9a-f]{8}$"),
    ({"length": 6, "charset": "digits"}, r"^[0-9]{6}$"),
]


@pytest.mark.parametrize("args,pattern", CASES)
async def test_random_string_matches_charset_shape(client, args, pattern):
    result = await client.call_tool("random_string", args)
    assert re.fullmatch(pattern, result.content[0].text)
