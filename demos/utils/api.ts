import fetch from "node-fetch";
import { Request } from "../types/types"

const COMPOSITOR_URL = "http://127.0.0.1:8081/--/api";

export async function sendAsync(body: Request): Promise<object> {
  let response;
  response = await fetch(COMPOSITOR_URL, {
    method: "POST",
    headers: {
      'Content-Type': 'application/json'
    },
    body: JSON.stringify(body),
  });

  if (response.status >= 400) {
    const err: any = new Error(`Request to compositor failed.`);
    err.status = await response.status;
    err.response = await response.json();
    throw err;
  }
  return await response.json();
}
