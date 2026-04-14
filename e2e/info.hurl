GET {{baseurl}}/info
HTTP 200
[Asserts]
jsonpath "$.std.name" == "random"
jsonpath "$.std.version" isString
