db = db.getSiblingDB("rinha");

db.createCollection("transactions");

db.transactions.createIndex({ client: 1 });
db.transactions.createIndex({ date: -1 });

// db.createCollection("clients", {
//   validator: {
//     $and: [
//       { $expr: { $gte: ["$balance.total", { $multiply: [-1, { $ifNull: ["$balance.limit", 0] }] }] } },
//       {
//         $jsonSchema: {
//           bsonType: "object",
//           properties: {
//             balance: {
//               bsonType: "object",
//               properties: {
//                 balance: {
//                   bsonType: "int",
//                 },
//                 limit: {
//                   bsonType: "int",
//                 },
//               },
//             },
//             latest_transactions: {
//               bsonType: "array",
//               items: {
//                 bsonType: "object",
//                 properties: {
//                   value: {
//                     bsonType: "int",
//                     minimum: 0,
//                   },
//                   kind: {
//                     enum: ["c", "d"],
//                   },
//                   description: {
//                     bsonType: "string",
//                     minLength: 1,
//                     maxLength: 10,
//                   },
//                   date: {
//                     bsonType: "date",
//                   },
//                 },
//               },
//             },
//           },
//         },
//       },
//     ],
//   },
// });

// db.clients.insertMany([
//   {
//     _id: 1,
//     balance: {
//       total: 0,
//       limit: 100000,
//     },
//     latest_transactions: [],
//   },
//   {
//     _id: 2,
//     balance: {
//       total: 0,
//       limit: 80000,
//     },
//     latest_transactions: [],
//   },
//   {
//     _id: 3,
//     balance: {
//       total: 0,
//       limit: 1000000,
//     },
//     latest_transactions: [],
//   },
//   {
//     _id: 4,
//     balance: {
//       total: 0,
//       limit: 10000000,
//     },
//     latest_transactions: [],
//   },
//   {
//     _id: 5,
//     balance: {
//       total: 0,
//       limit: 500000,
//     },
//     latest_transactions: [],
//   },
// ]);
