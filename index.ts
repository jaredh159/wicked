import fs from 'node:fs/promises';
import tf from '@tensorflow/tfjs-node';
import nsfw from 'nsfwjs';
import Fastify from 'fastify';

async function main(): Promise<void> {
  const model = await nsfw.load(); // todo: download model
  const app = Fastify({ logger: false });

  app.get(`/:filename`, async (request, reply) => {
    // @ts-ignore
    const filename: string = request.params.filename;
    if (!filename) {
      return reply.code(400).send(`Filename is required`);
    }

    try {
      var buffer = await getBuffer(filename);
    } catch (error) {
      return reply.code(400).send(`Error getting buffer ${error}`);
    }

    try {
      var image = await tf.node.decodeImage(buffer ?? Buffer.from([]), 3);
    } catch (error) {
      return reply.code(400).send(`Error decoding image ${error}`);
    }

    try {
      var predictions = await model.classify(image);
    } catch (error) {
      return reply.code(400).send(`Error classifying image ${error}`);
    }

    image.dispose();

    const results = {
      porn: predictions.find((p) => p.className === `Porn`)?.probability ?? 0,
      sexy: predictions.find((p) => p.className === `Sexy`)?.probability ?? 0,
      hentai: predictions.find((p) => p.className === `Hentai`)?.probability ?? 0,
    };
    reply.send(results);
  });

  try {
    await app.listen({ port: 8484 });
    process.stdout.write(`Listening on port 8484\n`);
  } catch (err) {
    process.exit(1);
  }
}

main();

async function getBuffer(filename: string): Promise<Buffer | undefined> {
  const filepath = `./${filename}`;
  try {
    const exists = await fs.exists(filepath);
    if (!exists) throw new Error();
  } catch (error) {
    process.stderr.write(`Filepath ${filepath} does not exist\n`);
    return undefined;
  }

  try {
    return await fs.readFile(filepath);
  } catch (error) {
    process.stderr.write(`Error reading file ${filepath}, ${error}\n`);
    return undefined;
  }
}
