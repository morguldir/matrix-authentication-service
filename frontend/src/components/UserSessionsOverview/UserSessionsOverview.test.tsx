// Copyright 2023 The Matrix.org Foundation C.I.C.
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

import { create } from "react-test-renderer";
import { describe, expect, it } from "vitest";

import { makeFragmentData } from "../../gql";
import { FRAGMENT as EMAIL_FRAGMENT } from "../UserEmail";

import UserSessionsOverview, { FRAGMENT } from "./";

describe("UserSessionsOverview", () => {
  it("render an simple <UserSessionsOverview />", () => {
    const primaryEmail = makeFragmentData(
      {
        id: "email:123",
        email: "hello@example.com",
        confirmedAt: new Date(),
      },
      EMAIL_FRAGMENT,
    );

    const user = makeFragmentData(
      {
        id: "user:123",
        primaryEmail: {
          id: "email:123",
          ...primaryEmail,
        },
        compatSessions: {
          totalCount: 0,
        },
        browserSessions: {
          totalCount: 0,
        },
        oauth2Sessions: {
          totalCount: 0,
        },
        unverifiedEmails: {
          totalCount: 0,
        },
        confirmedEmails: {
          totalCount: 1,
        },
      },
      FRAGMENT,
    );
    const component = create(<UserSessionsOverview user={user} />);
    const tree = component.toJSON();
    expect(tree).toMatchSnapshot();
  });

  it("render a <UserSessionsOverview /> with sessions", () => {
    const primaryEmail = makeFragmentData(
      {
        id: "email:123",
        email: "hello@example.com",
        confirmedAt: new Date(),
      },
      EMAIL_FRAGMENT,
    );

    const user = makeFragmentData(
      {
        id: "user:123",
        primaryEmail: {
          id: "email:123",
          ...primaryEmail,
        },
        compatSessions: {
          totalCount: 1,
        },
        browserSessions: {
          totalCount: 2,
        },
        oauth2Sessions: {
          totalCount: 3,
        },
        unverifiedEmails: {
          totalCount: 0,
        },
        confirmedEmails: {
          totalCount: 1,
        },
      },
      FRAGMENT,
    );
    const component = create(<UserSessionsOverview user={user} />);
    const tree = component.toJSON();
    expect(tree).toMatchSnapshot();
  });
});