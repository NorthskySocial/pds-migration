# <h1> PDS Migration Tool </h1>

[![License](https://img.shields.io/badge/license-MIT-blue)](https://opensource.org/licenses/mit)


## Overview
Backend Application that accepts HTTP calls allowing for a user to migrate from one PDS to another

HoppscotchCollection.json contains the necessary calls to make in order to perform migration

## Steps to Migrate (All HTTP calls are POST)
1. /service-auth
2. /create-account
3. /export-repo
4. /import-repo
5. /export-blobs
6. /upload-blobs
7. /migrate-preferences
8. /request-token
9. /migrate-plc
10. /activate-account
11. /deactivate-account