const API_BASE = '/api';


export interface CreatePollRequest {
	topic: string;
	description?: string;
}

export interface CreatePollResponse {
	success: boolean;
	message: string;
	uri?: string;
	cid?: string;
}

export interface CreateStatementRequest {
	text: string;
	poll_uri: string;
	poll_cid: string;
}

export interface CreateStatementResponse {
	success: boolean;
	message: string;
	uri?: string;
	cid?: string;
}

export interface CreateVoteRequest {
	value: 'agree' | 'disagree' | 'pass';
	statement_uri: string;
	statement_cid: string;
	poll_uri: string;
	poll_cid: string;
}

export interface CreateVoteResponse {
	success: boolean;
	message: string;
	uri?: string;
	cid?: string;
}


export async function createPoll(data: CreatePollRequest): Promise<CreatePollResponse> {
	const response = await fetch(`${API_BASE}/polls`, {
		method: 'POST',
		headers: { 'Content-Type': 'application/json' },
		body: JSON.stringify(data)
	});
	return response.json();
}

export async function createStatement(
	data: CreateStatementRequest
): Promise<CreateStatementResponse> {
	const response = await fetch(`${API_BASE}/statements`, {
		method: 'POST',
		headers: { 'Content-Type': 'application/json' },
		body: JSON.stringify(data)
	});
	return response.json();
}

export async function createVote(data: CreateVoteRequest): Promise<CreateVoteResponse> {
	console.log("Creating vote ", data)
	const response = await fetch(`${API_BASE}/votes`, {
		method: 'POST',
		headers: { 'Content-Type': 'application/json' },
		body: JSON.stringify(data)
	});
	return response.json();
}
