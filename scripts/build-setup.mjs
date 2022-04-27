#!/usr/bin/env zx

import 'zx/globals';
import fsExtra from 'fs-extra';
import fs from 'fs';
import os from 'os';
import path from 'path';
import jsYaml from 'js-yaml';
import inquirer from 'inquirer';
import { packageDirectory } from 'pkg-dir';
import { spawn } from "child_process";

process.env.FORCE_COLOR = 3
$.shell = '/bin/zsh'

const PKG_ROOT_DIR = await packageDirectory ();
const PROJECT_ROOT = path.resolve ( PKG_ROOT_DIR, '../' );
const ENV_YAML_FILE = `env.yaml`;
const DOT_ENV_FILE = `.env`;
const ENV_YAML_FILE_PATH = path.join ( PROJECT_ROOT, ENV_YAML_FILE );
const DOT_ENV_FILE_PATH = path.join ( PROJECT_ROOT, DOT_ENV_FILE );

async function getCmd ( cmd ) {
	const op = await $`${cmd}`;
	
	return op.stdout.trimEnd ();
}

function envMapToEnvStrings ( envMap ) {
	return Object.keys ( envMap ).map ( key => {
		const item = envMap[ key ];
		
		return `${key}="${item}"`;
	} )
}

async function getBuildEnvFromPrompt ( secretYamlFileDoc ) {
	const options = Object.keys ( secretYamlFileDoc.secrets );
	
	const output = await inquirer.prompt ( [
		{
			name: 'answer',
			message: 'Flashing environment?',
			type: 'list',
			choices: options,
		},
	] );
	
	console.info ( `Selected flashing environment: ${output.answer}` );
	
	return {
		answer: output.answer,
		data: secretYamlFileDoc.secrets[ output.answer ]
	}
}

async function runSpawn ( command, args ) {
	return new Promise ( ( resolve, reject ) => {
		const buildSpawn = spawn (
			command,
			args,
			{ stdio: [0, 1, 2] }
		);
		
		buildSpawn.on ( 'exit', ( exitCode ) => {
			if ( exitCode !== 0 ) {
				return reject ( exitCode );
			}
			
			return resolve ();
		} );
	} )
}

async function getDeviceType ( secretYamlFileDoc ) {
	const deviceIdList = secretYamlFileDoc.device_list.map ( a => a.DEVICE_ID );
	
	const output = await inquirer.prompt ( [
		{
			name: 'answer',
			message: 'Device Type?',
			type: 'list',
			choices: deviceIdList,
		},
	] );
	
	console.info ( `Selected device type: ${output.answer}` );
	
	return {
		answer: output.answer,
		data: secretYamlFileDoc.device_list.filter ( a => a.DEVICE_ID === output.answer )[ 0 ]
	}
}

const secretYamlFileDoc = jsYaml.load ( fsExtra.readFileSync ( ENV_YAML_FILE_PATH, 'utf8' ) );
const flashingEnvObj = await getBuildEnvFromPrompt ( secretYamlFileDoc );
const flashingDeviceType = await getDeviceType ( secretYamlFileDoc );

const flashingEnvStrings = envMapToEnvStrings ( flashingEnvObj.data );
const flashingDeviceTypeStrings = envMapToEnvStrings ( flashingDeviceType.data );
const flashingEnv = flashingEnvObj.answer;

await fsExtra.remove ( DOT_ENV_FILE_PATH )
await fsExtra.ensureFile ( DOT_ENV_FILE_PATH )

const dotFileId = fs.openSync ( DOT_ENV_FILE_PATH, 'a', 640 );
fs.writeSync ( dotFileId, `## This is a generated file${os.EOL}## DO NOT commit this file to GIT as it holds your secret keys${os.EOL}`, null, 'utf8' );
for ( const line of flashingEnvStrings ) {
	fs.writeSync ( dotFileId, `${line}${os.EOL}`, null, 'utf8' );
}
for ( const line of flashingDeviceTypeStrings ) {
	fs.writeSync ( dotFileId, `${line}${os.EOL}`, null, 'utf8' );
}
await fs.closeSync ( dotFileId );


if ( flashingEnv === 'release' ) {
	await runSpawn ( "cargo", ["build", "--release"] );
}
else {
	await runSpawn ( "cargo", ["build"] );
}

console.info ( `flashing the ${flashingEnvObj.answer} build..` );
console.info ( "hold boot button and release it when the flashing starts..." );

await $`sleep 2`

await runSpawn ( "espflash",
	["/dev/cu.usbserial-0001", `../target/xtensa-esp32-espidf/${flashingEnvObj.answer}/sirius-alpha-rust`] );

